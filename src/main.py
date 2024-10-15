#!/usr/bin/env python

from dotenv import load_dotenv
from discord import Bot
import discord
import json
from pathlib import Path
from os import getenv

class BingusBot(Bot):
    async def on_ready(self):
        print(f"Initialized Gateway as {self.user.name} ({self.user.id})")

load_dotenv()

print("Initializing Base Bot...")

intents = discord.Intents.default()
intents.message_content = True

bot = BingusBot(intents=intents)

EXTENSIONS: list[str] = json.loads(Path(__file__).parent.joinpath("cogs.json").read_text())

for ext in EXTENSIONS:
    print(f"Initializing \"{ext}\"...")
    bot.load_extension(ext)

print("Connecting to Discord...")

bot.run(getenv("TOKEN"))
