use std::sync::{Arc, atomic::Ordering};

use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::{
    application::interaction::{Interaction, application_command::CommandData},
    channel::Attachment,
};

use crate::{
    BotContext, brain::Brain, cmd::DEFER_INTER_RESP_EPHEMERAL, prelude::*, status::update_status,
};

#[derive(CommandModel, CreateCommand)]
#[command(name = "load_chain", desc = "Load a chain from a .msgpackz file")]
pub struct LoadChainCommand {
    /// Brain file to load
    file: Attachment,
    /// Use a Nushell (Legacy) compatible msgpack
    compat: Option<bool>,
}

impl LoadChainCommand {
    pub async fn handle(inter: Interaction, data: CommandData, ctx: Arc<BotContext>) -> Result {
        let Self { file, compat } =
            Self::from_interaction(data.into()).context("Failed to parse command data")?;

        let client = ctx.http.interaction(ctx.app_id);

        client
            .create_response(inter.id, &inter.token, &DEFER_INTER_RESP_EPHEMERAL)
            .await
            .context("Failed to defer")?;

        let mut data = std::io::Cursor::new(
            reqwest::get(file.url)
                .await
                .context("Failed to request attachment")?
                .bytes()
                .await
                .context("Failed to decode as bytes")?,
        );
        let mut brotli_stream = brotli::Decompressor::new(&mut data, 4096);

        let new_brain: Brain = if compat.unwrap_or_default() {
            Brain::from_legacy_hashmap(
                rmp_serde::from_read(&mut brotli_stream).context("Failed to decode brain file")?,
            )
        } else {
            rmp_serde::from_read(&mut brotli_stream).context("Failed to decode brain file")?
        };

        {
            let mut brain = ctx.brain_handle.lock().await;
            brain.merge_from(new_brain);
            ctx.pending_save.store(true, Ordering::Relaxed);
            update_status(&*brain, &ctx.shard_sender).context("Failed to update status")?;
        }

        client
            .update_response(&inter.token)
            .content(Some("Bingus Learned!"))
            .await
            .context("Failed to send brain")?;

        Ok(())
    }
}
