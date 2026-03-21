use std::sync::{Arc, atomic::Ordering};

use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::application::interaction::{Interaction, application_command::CommandData};

use crate::{
    BotContext, cmd::DEFER_INTER_RESP_EPHEMERAL, prelude::*, require_owner, status::update_status,
};

#[derive(CommandModel, CreateCommand)]
#[command(
    name = "forget",
    desc = "Erase a word from all edges. THIS ACTION IS IRREVERSIBLE!"
)]
pub struct ForgetCommand {
    /// The token to forget
    token: String,
}

impl ForgetCommand {
    pub async fn handle(inter: Interaction, data: CommandData, ctx: Arc<BotContext>) -> Result {
        let client = ctx.http.interaction(ctx.app_id);

        require_owner!(inter, ctx, client);

        let Self { token } =
            Self::from_interaction(data.into()).context("Failed to parse command data")?;

        client
            .create_response(inter.id, &inter.token, &DEFER_INTER_RESP_EPHEMERAL)
            .await
            .context("Failed to defer")?;

        {
            let mut brain = ctx.brain_handle.write().await;
            brain.forget(token.as_str());
            ctx.pending_save.store(true, Ordering::Relaxed);
            update_status(&brain, &ctx.shard_sender).context("Failed to update status")?;
        }

        client
            .update_response(&inter.token)
            .content(Some("Token forgotten"))
            .await
            .context("Failed to send brain")?;

        Ok(())
    }
}
