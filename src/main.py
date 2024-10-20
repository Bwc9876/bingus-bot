#!/usr/bin/env python

from dotenv import load_dotenv
from discord import Bot
import discord
import json
from pathlib import Path
from os import getenv
from lib.permissions import NotOwnerError

class BingusBot(Bot):

    def __init__(self, *args, **kwargs):
        self.bingus_owners = []
        super().__init__(*args, **kwargs)

    async def get_owners(self):
        app = await self.application_info()
        if app.team is not None:
            return [m.id for m in app.team.members]
        else:
            return [app.owner.id]
        
    async def on_application_command_error(self, context: discord.ApplicationContext, exception: discord.DiscordException) -> None:
        if isinstance(exception, NotOwnerError):
            await context.respond("You are not allowed to run this command!", ephemeral=True)
        else:
            return await super().on_application_command_error(context, exception)

    async def on_ready(self):
        self.bingus_owners = await self.get_owners()
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
