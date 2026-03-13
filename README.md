# Bingus Bot

A very clever kitty

Simple markov chain-based auto replies to user messages.

## Installation

Use the NixOS module in this flake or build/set it up yourself. Just need Rust.

## Configuration

### `TOKEN_FILE`

Path to the file containing the bot's token.

### `BRAIN_FILE`

Path to the file Bingus will save state in. The file will be created if not present. This will be a brotli-compressed MsgPack file.

### `REPLY_CHANNELS`

Comma-delimited list of channel IDs that Bingus should auto reply in.

Bingus will learn from all channels he has access to but will only auto-reply in these ones.

## Commands

Commands with `*` require you to be the owner of the bot. Create a team in the Discord
dev portal to make multiple people owner.

- `/markov`: For a reply from Bingus in the current channel
- `/weights`: View the weights for a specific token
- `/dump_chain`: Dump Bingus' "brain", his entire database of known words and relations
- `/load_chain`: Additively load a brain file into Bingus

