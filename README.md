# Bingus Bot

A very clever kitty

Simple markov chain-based auto replies to user messages.

## Installation

Use the NixOS module in this flake or build/set it up yourself. Just need Rust.

### Discord Setup

This bot will need the "Message Content" privileged gateway intent to learn from messages.

It will also need the ability to send messages and read the message history of channels.

## Configuration

### `TOKEN_FILE`

Path to the file containing the bot's token.

### `BRAIN_FILE`

Path to the file Bingus will save state in. The file will be created if not present. This will be a brotli-compressed MsgPack file.

### `REPLY_CHANNELS`

Comma-delimited list of channel IDs that Bingus should auto reply in.

Bingus will learn from all channels he has access to but will only auto-reply in these ones.

To retrieve this ID, enable developer mode in your client and right click the channel you wish to use, then click "Copy Channel ID".

## Commands

Commands with `*` require you to be the owner of the bot. Create a team in the Discord
dev portal to make multiple people owner.

- `/markov`: For a reply from Bingus in the current channel
- `/weights`: View the weights for a specific token
- *`/forget`: Clear a word from memory, this gets rid of any instance of the token
- *`/dump_chain`: Dump Bingus' "brain", his entire database of known words and relations
- *`/load_chain`: Additively load a brain file into Bingus

