#![feature(iter_map_windows)]
#![feature(iter_intersperse)]

mod brain;
mod cmd;
mod on_message;
mod status;

pub mod prelude {
    pub use anyhow::Context;
    use std::result::Result as StdResult;
    pub type Result<T = (), E = anyhow::Error> = StdResult<T, E>;
}

use std::{
    collections::HashSet,
    fs::File,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use brotli::enc::BrotliEncoderParams;
use log::{debug, error, info, warn};
use prelude::*;
use tokio::{
    sync::Mutex,
    time::{self, Duration},
};
use twilight_gateway::{
    CloseFrame, Event, EventTypeFlags, Intents, MessageSender, Shard, ShardId, StreamExt,
};
use twilight_http::Client as HttpClient;
use twilight_model::{
    application::interaction::InteractionData,
    id::{
        Id,
        marker::{ApplicationMarker, UserMarker},
    },
};

use crate::{
    brain::Brain,
    cmd::{handle_app_command, register_all_commands},
    on_message::handle_discord_message,
    status::update_status,
};

pub type BrainHandle = Mutex<Brain>;

#[derive(Debug)]
pub struct BotContext {
    http: HttpClient,
    self_id: Id<UserMarker>,
    app_id: Id<ApplicationMarker>,
    brain_file_path: PathBuf,
    reply_channels: HashSet<u64>,
    brain_handle: BrainHandle,
    shard_sender: MessageSender,
    pending_save: AtomicBool,
}

async fn handle_discord_event(event: Event, ctx: Arc<BotContext>) -> Result {
    match event {
        Event::MessageCreate(msg) => handle_discord_message(msg, ctx).await,
        Event::InteractionCreate(mut inter) => {
            if let Some(InteractionData::ApplicationCommand(data)) =
                std::mem::take(&mut inter.0.data)
            {
                handle_app_command(*data, ctx, inter.0).await
            } else {
                Ok(())
            }
        }
        Event::Ready(ev) => {
            info!("Connected to gateway as {}", ev.user.name);
            let brain = ctx.brain_handle.lock().await;
            update_status(&*brain, &ctx.shard_sender).context("Failed to update status")
        }
        _ => Ok(()),
    }
}

fn load_brain(path: &Path) -> Result<Option<Brain>> {
    if path.exists() {
        let mut file = File::open(path).context("Failed to open brain file")?;
        let mut brotli_stream = brotli::Decompressor::new(&mut file, 4096);
        rmp_serde::from_read(&mut brotli_stream)
            .map(|b| Some(b))
            .context("Failed to decode brain file")
    } else {
        Ok(None)
    }
}

async fn save_brain(ctx: Arc<BotContext>) -> Result {
    let mut file = File::create(&ctx.brain_file_path).context("Failed to open brain file")?;
    let params = BrotliEncoderParams::default();
    let mut brotli_writer = brotli::CompressorWriter::with_params(&mut file, 4096, &params);
    let brain = ctx.brain_handle.lock().await;
    rmp_serde::encode::write(&mut brotli_writer, &*brain)
        .context("Failed to write serialized brain")?;
    debug!("Saved brain file");
    Ok(())
}

#[tokio::main]
async fn main() -> Result {
    let mut clog = colog::default_builder();
    clog.filter(
        None,
        if cfg!(debug_assertions) {
            log::LevelFilter::Debug
        } else {
            log::LevelFilter::Info
        },
    );
    clog.try_init().context("Failed to initialize colog")?;

    info!("Start of bingus-bot {}", env!("CARGO_PKG_VERSION"));

    // Config
    let token_file = std::env::var("TOKEN_FILE").context("Missing TOKEN_FILE env var")?;
    let reply_channels: HashSet<u64> = std::env::var("REPLY_CHANNELS")
        .context("Missing REPLY_CHANNELS env var")?
        .split(",")
        .map(|s| s.trim().parse::<u64>())
        .collect::<Result<_, _>>()
        .context("Invalid channel IDs for REPLY_CHANNELS")?;
    let brain_file_path =
        PathBuf::from(std::env::var("BRAIN_FILE").unwrap_or_else(|_| "brain.msgpackz".to_string()));
    let intents = Intents::GUILD_MESSAGES | Intents::MESSAGE_CONTENT;

    // Read token
    let token = std::fs::read_to_string(token_file).context("Failed to read bot token")?;
    let token = token.trim();

    // Read Brain
    let brain = if let Some(brain) = load_brain(&brain_file_path)? {
        info!("Loading brain from {brain_file_path:?}");
        brain
    } else {
        info!("Creating new brain file at {brain_file_path:?}");
        Brain::default()
    };
    let brain_handle = Mutex::new(brain);

    // Init
    let mut shard = Shard::new(ShardId::ONE, token.to_string(), intents);
    let http = HttpClient::new(token.to_string());

    let app = http
        .current_user_application()
        .await
        .context("Failed to get current App")?
        .model()
        .await
        .context("Failed to deserialize")?;

    let app_id = app.id;

    let self_id = app.bot.context("App is not a bot!")?.id;

    let context = Arc::new(BotContext {
        http,
        self_id,
        app_id,
        reply_channels,
        brain_file_path,
        brain_handle,
        shard_sender: shard.sender(),
        pending_save: AtomicBool::new(false),
    });

    info!("Ensuring brain is writable...");
    save_brain(context.clone())
        .await
        .context("Brain file is not writable")?;
    info!("Brain file saved");

    info!("Registering Commands...");
    register_all_commands(context.clone()).await?;

    let mut interval = time::interval(Duration::from_secs(60));
    interval.tick().await;
    tokio::pin!(interval);

    info!("Connecting to gateway...");

    loop {
        tokio::select! {

            biased;

            Ok(()) = tokio::signal::ctrl_c() => {
                info!("SIGINT: Closing connection and saving");
                shard.close(CloseFrame::NORMAL);
                break;
            }
            _ = interval.tick() => {
                debug!("Save Interval");
                if context.pending_save.load(Ordering::Relaxed) {
                   let ctx = context.clone();
                   tokio::spawn(async move {
                        if let Err(why) = save_brain(ctx.clone()).await {
                            error!("Failed to save brain file:\n{why:?}");
                        }
                        ctx.pending_save.store(false, Ordering::Relaxed);
                   });
                }
            },
            opt = shard.next_event(EventTypeFlags::all()) => {
                match opt {
                    Some(Ok(Event::GatewayClose(_))) | None => {
                        info!("Disconnected from Discord: Saving brain and exiting");
                        break;
                    }
                    Some(Ok(event)) => {
                        let ctx = context.clone();
                        tokio::spawn(async move {
                            if let Err(why) = handle_discord_event(event, ctx).await {
                                error!("Error while processing Discord event:\n{why:?}");
                            }
                        });
                    }
                    Some(Err(why)) => {
                        warn!("Failed to receive event:\n{why:?}");
                    }
                }
            }
        }
    }

    save_brain(context)
        .await
        .context("Failed to write brain file on exit")?;

    info!("Save Complete, Exiting");

    Ok(())
}
