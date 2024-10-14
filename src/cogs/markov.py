
import random
import os
import discord
from discord.ext import commands
from discord.message import Message
from lib.markov import MarkovChain

class Markov(commands.Cog):

    def __init__(self, bot):
        self.bot = bot
        self.reply_channels = [int(x) for x in os.getenv("REPLY_CHANNELS", "0").split(",")]
        self.markov = MarkovChain({})

    @commands.slash_command()
    async def scan_history(self, ctx):
        buf = []
        await ctx.defer()
        async for msg in ctx.history(limit=None):
            print("Learning...")
            if msg.author.id != self.bot.application_id:
                buf.append(msg.content)
        self.markov.learn(" ".join(buf))
        await ctx.respond("Bingus Learned!")

    @commands.slash_command()
    async def markov(self, ctx, prompt: discord.Option(str)):
        print("Bingus is responding!")
        response = self.markov.respond(prompt)
        if response is not None and len(response) != 0:
            await ctx.respond(response)
        else:
            await ctx.respond(":: Bingus couldn't think of what to say!")
        print("Bingus Responded!")


    @commands.Cog.listener() # we can add event listeners to our cog
    async def on_message(self, msg: Message):
        if msg.author.id != self.bot.application_id:
            print("Bingus is learning!")
            self.markov.learn(msg.content)

        chance = 80 if msg.author.id != self.bot.application_id else 35

        if msg.channel.id in self.reply_channels and random.randint(1, 100) <= chance:
            print("Bingus is responding!")
            response = self.markov.respond(msg.content)
            if response is not None and len(response) != 0:
                await msg.channel.send(response)
            print("Bingus Responded!")

def setup(bot):
    bot.add_cog(Markov(bot))
