#!/usr/bin/env python

from dotenv import load_dotenv
from discord import Bot
import discord
from os import getenv

class BingusBot(Bot):
    async def on_ready(self):
        print(f"Initialized Gateway as {self.user.name} ({self.user.id})")

load_dotenv()

print("Initializing...")

intents = discord.Intents.default()
intents.message_content = True

bot = BingusBot(intents=intents)

bot.load_extension("cogs.markov")

bot.run(getenv("TOKEN"))
