#![feature(iter_map_windows)]
#![allow(unused)]

mod brain;

pub mod prelude {
    pub use anyhow::Context;
    use std::result::Result as StdResult;
    pub type Result<T = (), E = anyhow::Error> = StdResult<T, E>;
}

use std::{collections::HashSet, sync::Arc};

use prelude::*;
use twilight_cache_inmemory::{DefaultInMemoryCache, ResourceType};
use twilight_gateway::{Event, EventTypeFlags, Intents, Shard, ShardId, StreamExt};
use twilight_http::Client as HttpClient;

#[derive(Debug)]
struct BotContext {
    http: HttpClient,
    reply_channels: HashSet<String>,
}

async fn handle_discord_event(event: Event, _ctx: Arc<BotContext>) -> Result {
    match event {
        Event::MessageCreate(msg) => {
            let channel_id = msg.channel_id.to_string();
            eprintln!("id: {channel_id}");
        }
        Event::Ready(ev) => {
            eprintln!("Connected to gateway as {}", ev.user.name);
        }
        _ => {}
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result {
    // Config
    let token_file = std::env::var("TOKEN_FILE").context("Missing TOKEN_FILE env var")?;
    let reply_channels: HashSet<String> = HashSet::from_iter(
        std::env::var("REPLY_CHANNELS")
            .context("Missing REPLY_CHANNELS env var")?
            .split(",")
            .map(|s| s.trim().to_string()),
    );
    let intents = Intents::GUILD_MESSAGES | Intents::MESSAGE_CONTENT;

    // Read token
    let token = std::fs::read_to_string(token_file).context("Failed to read bot token")?;
    let token = token.trim();

    // Init
    let mut shard = Shard::new(ShardId::ONE, token.to_string(), intents);
    let http = HttpClient::new(token.to_string());
    let cache = DefaultInMemoryCache::builder()
        .resource_types(
            ResourceType::MESSAGE
                | ResourceType::USER
                | ResourceType::CHANNEL
                | ResourceType::USER_CURRENT,
        )
        .build();

    let context = Arc::new(BotContext {
        http,
        reply_channels,
    });

    // Event Loop
    while let Some(res) = shard.next_event(EventTypeFlags::all()).await {
        match res {
            Ok(event) => {
                cache.update(&event);
                tokio::spawn(handle_discord_event(event, Arc::clone(&context)));
            }
            Err(why) => {
                eprintln!("Failed to receive event: {why:?}");
            }
        }
    }

    Ok(())
}
