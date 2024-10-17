# Bingus Bot

A very clever kitty

Current features:

- [Markov chain](https://en.wikipedia.org/wiki/Markov_chain) for a "smart" chatbot

## Pre-reqs

- Python 3.12
- Pipenv
- Just (optional, QoL)

## Environment Setup

With `just`: `just setup`

Otherwise, `pipenv install`

### .env

Minimal needed is an env var called `TOKEN` that contains the bot's token.

Cogs may have additional options, these variables **should** be prefix with the cog's name, ex:

Cog `Math` may have `Math.TEMP_UNIT=fahrenheit`.

## Cog Docs

### [Markov](src/cogs/markov.py)

A markov chain is an incredibly simple model where it decides the next token based on previous
knowledge of what tokens the current token has been proceeded by.

#### Markov Commands

- `/markov`: Make bingus try and reply to a prompt passed, use this to bypass the 80% change that bingus
  usually has to reply
- `/scan_history`: Scan the history of the current channel and add it to the chain. Since Bingus only learns
  from *new* messages while he's active, you may need to do this when restarting him. This command can take a while depending on the number of messages.
- `/weights`: Dump the weights of the specified token to other tokens

#### Markov Config

- `Markov.REPLY_CHANNELS`: A *comma-delimited* list of channel IDs that the bot should have
  have a chance to reply to messages in. The bot still learns from all channels in realtime, but
  these channels it'll have an 80% of replying to any message

## Adding Cogs

To start, you can run `just add-cog` (or `python src/dev/new_cog.py`).

Follow the steps and you'll get a new cog in `src/cogs`. Here is where you can add
commands, event listeners, etc.

See the file generated for some simple examples, review the [PyCord docs](https://guide.pycord.dev/introduction) for more help and information.

### Best Practices

Generally you'll want to make all state, config, etc. locallized to your cog file. That
way we can easily disable it if needed and nothing will break. 

#### Config

For simplicity we'll just use env vars for config. Prefix any env vars your cog will
read with the cog's name and a `.` (example: `Markov` cog can have a var called `Markov.REPLY_CHANNELS`).

Try to documents these options within this README file under the [The individual cogs docs](#cog-docs).
Create a third-level heading with your cog's name, and a link to its source code.
