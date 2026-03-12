mod weights;

use std::sync::Arc;

use log::warn;
use twilight_interactions::command::CreateCommand;
use twilight_model::application::interaction::{Interaction, application_command::CommandData};
use twilight_model::http::interaction::{InteractionResponse, InteractionResponseType};

use crate::{BotContext, prelude::*};

use weights::WeightsCommand;

const DEFER_INTER_RESP: InteractionResponse = InteractionResponse {
    kind: InteractionResponseType::DeferredChannelMessageWithSource,
    data: None,
};

pub async fn register_all_commands(ctx: Arc<BotContext>) -> Result {
    let commands = [WeightsCommand::create_command().into()];

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
        other => {
            warn!("Unknown command send: {other}");
            Ok(())
        }
    }
}
