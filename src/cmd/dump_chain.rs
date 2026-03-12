use std::sync::Arc;

use brotli::enc::BrotliEncoderParams;
use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::{
    application::interaction::{Interaction, application_command::CommandData},
    http::attachment::Attachment,
};

use crate::{BotContext, cmd::DEFER_INTER_RESP_EPHEMERAL, prelude::*};

#[derive(CommandModel, CreateCommand)]
#[command(name = "dump_chain", desc = "Dump chain")]
pub struct DumpChainCommand {
    /// Generate as Nushell (Legacy) compatible msgpack
    compat: Option<bool>,
}

impl DumpChainCommand {
    pub async fn handle(inter: Interaction, data: CommandData, ctx: Arc<BotContext>) -> Result {
        let Self { compat } =
            Self::from_interaction(data.into()).context("Failed to parse command data")?;

        let client = ctx.http.interaction(ctx.app_id);

        client
            .create_response(inter.id, &inter.token, &DEFER_INTER_RESP_EPHEMERAL)
            .await
            .context("Failed to defer")?;

        let mut buf = Vec::<u8>::with_capacity(4096);
        let params = BrotliEncoderParams::default();
        let mut brotli_writer = brotli::CompressorWriter::with_params(&mut buf, 4096, &params);

        if compat.unwrap_or_default() {
            let brain = ctx.brain_handle.lock().await;
            let map = brain.as_legacy_hashmap();
            drop(brain);
            rmp_serde::encode::write(&mut brotli_writer, &map)
                .context("Failed to legacy encode brain")?;
        } else {
            let brain = ctx.brain_handle.lock().await;
            rmp_serde::encode::write(&mut brotli_writer, &*brain)
                .context("Failed to write serialized brain")?;
        }
        drop(brotli_writer);

        let attachment = Attachment::from_bytes(String::from("brain.msgpackz"), buf, 1);

        client
            .update_response(&inter.token)
            .attachments(&[attachment])
            .await
            .context("Failed to send brain")?;

        Ok(())
    }
}
