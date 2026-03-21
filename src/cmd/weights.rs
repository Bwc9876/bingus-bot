use std::sync::Arc;

use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::{
    application::interaction::{Interaction, application_command::CommandData},
    http::attachment::Attachment,
};

use crate::{BotContext, BrainHandle, brain::format_token, cmd::DEFER_INTER_RESP, prelude::*};

#[derive(CommandModel, CreateCommand)]
#[command(name = "weights", desc = "Get the weights of a token")]
pub struct WeightsCommand {
    /// Token to view the weights of
    token: String,
}

async fn get_output(token: &str, brain: &BrainHandle) -> Option<String> {
    let brain = brain.read().await;

    brain.get_weights(token).map(|edges| {
        let sep = String::from("\n");
        let mut all_weights = edges.iter_weights().collect::<Vec<_>>();

        all_weights.sort_by_key(|(_, w, _)| *w);

        let formatted_weights = all_weights
            .into_iter()
            .map(|(token, weight, chance)| {
                let token_fmt = format_token(token);
                format!("{token_fmt}: {:.1}% ({weight})", chance * 100.0)
            })
            .intersperse(sep)
            .collect::<String>();

        format!("Weights for {token}:\n{formatted_weights}")
    })
}

impl WeightsCommand {
    pub async fn handle(inter: Interaction, data: CommandData, ctx: Arc<BotContext>) -> Result {
        let Self { token } =
            Self::from_interaction(data.into()).context("Failed to parse command data")?;

        let client = ctx.http.interaction(ctx.app_id);

        client
            .create_response(inter.id, &inter.token, &DEFER_INTER_RESP)
            .await
            .context("Failed to defer")?;

        let content = get_output(&token, &ctx.brain_handle)
            .await
            .unwrap_or_else(|| String::from("Bingus doesn't know that word!"));

        let update = client.update_response(&inter.token);

        if content.encode_utf16().count() < 2000 {
            update.content(Some(content.as_str())).await
        } else {
            let data = content.into_bytes();
            let attachment = Attachment::from_bytes(String::from("weights.txt"), data, 1);
            update
                .content(Some(
                    "Weights were too long to fit into one message, check the text file!",
                ))
                .attachments(&[attachment])
                .await
        }
        .context("Failed to reply")?;

        Ok(())
    }
}
