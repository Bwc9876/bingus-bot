use std::sync::Arc;

use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::application::interaction::{Interaction, application_command::CommandData};

use crate::{BotContext, cmd::DEFER_INTER_RESP, prelude::*};

#[derive(CommandModel, CreateCommand)]
#[command(
    name = "markov",
    desc = "Trigger a response from bingus! Uses the last word you sent to start the chain"
)]
pub struct MarkovCommand {
    /// Prompt bingus should reply to
    prompt: String,
}

impl MarkovCommand {
    pub async fn handle(inter: Interaction, data: CommandData, ctx: Arc<BotContext>) -> Result {
        let Self { prompt } =
            Self::from_interaction(data.into()).context("Failed to parse command data")?;

        let client = ctx.http.interaction(ctx.app_id);

        client
            .create_response(inter.id, &inter.token, &DEFER_INTER_RESP)
            .await
            .context("Failed to defer")?;

        let brain = ctx.brain_handle.read().await;
        let content = brain
            .respond(&prompt, false, true, None)
            .unwrap_or_else(|| String::from("> Bingus couldn't think of what to say!"));
        drop(brain);

        client
            .update_response(&inter.token)
            .content(Some(content.as_str()))
            .await
            .context("Failed to reply")?;

        Ok(())
    }
}
