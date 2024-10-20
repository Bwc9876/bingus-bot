
import random
import os
import io
import discord
from discord.ext import commands
from discord.message import Message
from lib.markov import MarkovChain
from lib.permissions import require_owner

class Markov(commands.Cog):

    def __init__(self, bot: discord.bot.Bot):
        self.bot = bot
        self.reply_channels = [int(x) for x in os.getenv("Markov.REPLY_CHANNELS", "0").split(",")]
        self.markov = MarkovChain({})

    async def update_words(self):
        amount = len(self.markov.edges.keys())
        await self.bot.change_presence(activity=discord.CustomActivity(name=f"I know {amount} words!"))

    @require_owner
    @commands.slash_command()
    async def dump_chain(self, ctx: discord.ApplicationContext):
        o = self.markov.dump()
        fd = io.BytesIO(o.encode())
        await ctx.respond(ephemeral=True, file=discord.File(fd, filename="brain.json"))

    @require_owner
    @commands.slash_command()
    async def load_chain(self, ctx: discord.ApplicationContext, raw: discord.Option(discord.Attachment)):
        j = (await raw.read()).decode("utf-8")
        new = MarkovChain.load(j)
        self.markov.merge(new)
        await ctx.respond("Imported", ephemeral=True)
        await self.update_words()

    @require_owner
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
            head = f"Weights for **{token}**"
            msg = '\n'.join([f"{str(k)}: {v}" for k, v in edges.items()])
            if len(msg) > 1750:
                fd = io.BytesIO(msg.encode())
                await ctx.respond(head, file=discord.File(fd, filename="weights.txt"))
            else:
                await ctx.respond(f"{head}:\n{msg}")

    @commands.Cog.listener()
    async def on_message(self, msg: Message):

        if msg.flags.ephemeral:
            return

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
