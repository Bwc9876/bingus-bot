use std::sync::{Arc, atomic::Ordering};

use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::application::interaction::{Interaction, application_command::CommandData};

use crate::{
    BotContext, cmd::DEFER_INTER_RESP_EPHEMERAL, prelude::*, require_owner, status::update_status,
};

#[derive(CommandModel, CreateCommand)]
#[command(
    name = "forget-edge",
    desc = "Erase a specific edge in the graph. THIS ACTION IS IRREVERSIBLE!"
)]
pub struct ForgetEdgeCommand {
    /// From token
    from: String,
    /// To token
    to: String,
}

impl ForgetEdgeCommand {
    pub async fn handle(inter: Interaction, data: CommandData, ctx: Arc<BotContext>) -> Result {
        let client = ctx.http.interaction(ctx.app_id);

        require_owner!(inter, ctx, client);

        let Self { from, to } =
            Self::from_interaction(data.into()).context("Failed to parse command data")?;

        client
            .create_response(inter.id, &inter.token, &DEFER_INTER_RESP_EPHEMERAL)
            .await
            .context("Failed to defer")?;

        let existed = {
            let mut brain = ctx.brain_handle.write().await;
            let existed = brain.forget_edge(&from, &to);
            if existed {
                ctx.pending_save.store(true, Ordering::Relaxed);
                update_status(&brain, &ctx.shard_sender).context("Failed to update status")?;
            }
            existed
        };

        let msg = if existed {
            "Edge forgotten"
        } else {
            "That edge does not seem to exist"
        };

        client
            .update_response(&inter.token)
            .content(Some(msg))
            .await
            .context("Failed to send brain")?;

        Ok(())
    }
}
