use std::{
    boxed::Box,
    sync::{Arc, atomic::Ordering},
};

use log::warn;
use twilight_model::{
    channel::message::{AllowedMentions, MessageFlags, MessageType},
    gateway::payload::incoming::MessageCreate,
};

use crate::{BotContext, prelude::*, status::update_status};

pub async fn handle_discord_message(msg: Box<MessageCreate>, ctx: Arc<BotContext>) -> Result {
    let channel_id = msg.channel_id.get();
    let is_self = msg.author.id == ctx.self_id;
    let is_normal_message = matches!(msg.kind, MessageType::Regular | MessageType::Reply);
    let is_ephemeral = msg
        .flags
        .is_some_and(|flags| flags.contains(MessageFlags::EPHEMERAL));
    let is_dm = msg.guild_id.is_none();

    // Should we consider this message at all?
    if !is_normal_message || is_ephemeral || is_dm {
        return Ok(());
    }

    // Should we learn from this message? (We don't want to learn from ourselves)
    if !is_self {
        let mut brain = ctx.brain_handle.lock().await;
        let learned_new_word = brain.ingest(&msg.content);
        ctx.pending_save.store(true, Ordering::Relaxed);

        if learned_new_word {
            update_status(&*brain, &ctx.shard_sender).context("Failed to update status")?;
        }
    }

    // Should Reply to Message?
    if !ctx.reply_channels.contains(&channel_id) {
        return Ok(());
    }

    let (typ_tx, typ_rx) = tokio::sync::oneshot::channel();
    let (done_tx, done_rx) = tokio::sync::oneshot::channel();

    let ctx_typ = ctx.clone();
    let typ_id = msg.channel_id;
    tokio::spawn(async move {
        if typ_rx.await.ok().is_some_and(|start| start) {
            if let Err(why) = ctx_typ.http.create_typing_trigger(typ_id).await {
                warn!("Failed to set typing indicator:\n{why:?}");
            }
        }
        done_tx.send(()).ok();
    });

    let brain = ctx.brain_handle.lock().await;
    if let Some(reply_text) = brain
        .respond(&msg.content, is_self, Some(typ_tx))
        .filter(|s| !s.trim().is_empty())
    {
        drop(brain);
        done_rx.await.ok();
        let allowed_mentions = AllowedMentions::default();
        let my_msg = ctx
            .http
            .create_message(msg.channel_id)
            .content(&reply_text)
            .allowed_mentions(Some(&allowed_mentions));

        let my_msg = if !is_self {
            my_msg.reply(msg.id).fail_if_not_exists(false)
        } else {
            my_msg
        };

        my_msg.await.context("Failed to send message")?;
    }

    Ok(())
}
