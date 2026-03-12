mod dump_chain;
mod load_chain;
mod markov;
mod weights;

use std::sync::Arc;

use log::warn;
use twilight_interactions::command::CreateCommand;
use twilight_model::application::interaction::{Interaction, application_command::CommandData};
use twilight_model::channel::message::MessageFlags;
use twilight_model::http::interaction::{
    InteractionResponse, InteractionResponseData, InteractionResponseType,
};

use crate::{BotContext, prelude::*};

use dump_chain::DumpChainCommand;
use load_chain::LoadChainCommand;
use markov::MarkovCommand;
use weights::WeightsCommand;

const DEFER_INTER_RESP: InteractionResponse = InteractionResponse {
    kind: InteractionResponseType::DeferredChannelMessageWithSource,
    data: None,
};

const DEFER_INTER_RESP_EPHEMERAL: InteractionResponse = InteractionResponse {
    kind: InteractionResponseType::DeferredChannelMessageWithSource,
    data: Some(InteractionResponseData {
        allowed_mentions: None,
        attachments: None,
        choices: None,
        components: None,
        content: None,
        custom_id: None,
        embeds: None,
        flags: Some(MessageFlags::EPHEMERAL),
        title: None,
        tts: None,
        poll: None,
    }),
};

#[macro_export]
macro_rules! require_owner {
    ($inter:expr, $ctx:expr, $client:expr) => {
        if $inter.author_id().is_none_or(|id| !$ctx.owners.contains(&id)) {
            let data = twilight_util::builder::InteractionResponseDataBuilder::new()
                        .content("You're not allowed to run this command!")
                        .flags(twilight_model::channel::message::MessageFlags::EPHEMERAL)
                        .build();
            let resp = twilight_model::http::interaction::InteractionResponse {
                kind: twilight_model::http::interaction::InteractionResponseType::ChannelMessageWithSource,
                data: Some(data),
            };
            $client.create_response($inter.id, &$inter.token, &resp).await.context("Failed to deny perms")?;
            return Ok(());
        }
    };
}

pub async fn register_all_commands(ctx: Arc<BotContext>) -> Result {
    let commands = [
        WeightsCommand::create_command().into(),
        DumpChainCommand::create_command().into(),
        LoadChainCommand::create_command().into(),
        MarkovCommand::create_command().into(),
    ];

    let client = ctx.http.interaction(ctx.app_id);

    client
        .set_global_commands(&commands)
        .await
        .context("Failed to register app commands")?;

    Ok(())
}

pub async fn handle_app_command(
    data: CommandData,
    ctx: Arc<BotContext>,
    inter: Interaction,
) -> Result {
    match &*data.name {
        "weights" => WeightsCommand::handle(inter, data, ctx).await,
        "dump_chain" => DumpChainCommand::handle(inter, data, ctx).await,
        "load_chain" => LoadChainCommand::handle(inter, data, ctx).await,
        "markov" => MarkovCommand::handle(inter, data, ctx).await,
        other => {
            warn!("Unknown command send: {other}");
            Ok(())
        }
    }
}
