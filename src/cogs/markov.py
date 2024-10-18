
import random
import os
import discord
from discord.ext import commands
from discord.message import Message
from lib.markov import MarkovChain

class Markov(commands.Cog):

    def __init__(self, bot: discord.bot.Bot):
        self.bot = bot
        self.reply_channels = [int(x) for x in os.getenv("Markov.REPLY_CHANNELS", "0").split(",")]
        self.markov = MarkovChain({})

    async def update_words(self):
        amount = len(self.markov.edges.keys())
        await self.bot.change_presence(activity=discord.CustomActivity(name=f"I know {amount} words!"))

    @commands.slash_command()
    async def scan_history(self, ctx: discord.ApplicationContext):
        await ctx.defer()
        async for msg in ctx.history(limit=None):
            if msg.author.id != self.bot.application_id:
                self.markov.learn(msg.content)
        await ctx.respond("> Bingus Learned!")
        await self.update_words()

    @commands.slash_command()
    async def markov(self, ctx: discord.ApplicationContext, prompt: discord.Option(str)):
        print("Bingus is responding!")
        response = self.markov.respond(prompt)
        if response is not None and len(response) != 0:
            await ctx.respond(f"{prompt} {response[:1000]}")
        else:
            await ctx.respond("> Bingus couldn't think of what to say!")
        print("Bingus Responded!")

    @commands.slash_command()
    async def weights(self, ctx: discord.ApplicationContext, token: discord.Option(str)):
        edges = self.markov.get_edges(token)
        if edges is None:
            await ctx.respond("> Bingus doesn't know that word!")
        else:
            msg = '\n'.join([f"**{str(k)}**: {v}" for k, v in edges.items()])
            await ctx.respond(f"Weights for **{token}**:\n{msg}")

    @commands.Cog.listener()
    async def on_message(self, msg: Message):
        if msg.author.id != self.bot.application_id:
            print("Bingus is learning!")
            self.markov.learn(msg.content)
            await self.update_words()

        chance = 80 if msg.author.id != self.bot.application_id else 45

        if msg.channel.id in self.reply_channels and random.randint(1, 100) <= chance:
            print("Bingus is responding!")
            response = self.markov.respond(msg.content)
            if response is not None and len(response) != 0:
                await msg.channel.trigger_typing()
                if msg.author.id == self.bot.application_id:
                    await msg.channel.send(response[:1000])
                else:
                    await msg.reply(response[:1000], mention_author=False)
            print("Bingus Responded!")

def setup(bot):
    bot.add_cog(Markov(bot))
