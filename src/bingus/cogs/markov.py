import random
import os
import io
import discord
import pytesseract
import PIL
from discord.ext import commands
from discord.message import Message
from pathlib import Path
from ..lib.markov import MarkovChain
from ..lib.permissions import require_owner


class Markov(commands.Cog):
    def __init__(self, bot: discord.bot.Bot):
        self.bot = bot
        self.reply_channels = [
            int(x) for x in os.getenv("Markov.REPLY_CHANNELS", "0").split(",")
        ]
        self.chain_file = Path(os.getenv("Markov.BRAIN_FILE", "brain.msgpackz"))
        if self.chain_file.is_file():
            print(f"Attempting load from {self.chain_file}...")
            try:
                self.markov = MarkovChain.load_from_file(self.chain_file)
                print("Load Complete")
            except Exception as E:
                print(f"Error while loading\n{E}")
        else:
            self.markov = MarkovChain({})

    async def update_words(self):
        amount = len(self.markov.edges.keys())
        try:
            self.markov.save_to_file(self.chain_file)
        except Exception as E:
            print(f"Error while saving\n{E}")

        await self.bot.change_presence(
            activity=discord.CustomActivity(name=f"I know {amount} words!")
        )

    @require_owner
    @commands.slash_command()
    async def dump_chain(self, ctx: discord.ApplicationContext):
        o = self.markov.dumpb()
        fd = io.BytesIO(o)
        await ctx.respond(
            ephemeral=True, file=discord.File(fd, filename="brain.msgpackz")
        )

    @require_owner
    @commands.slash_command()
    async def load_chain(
        self, ctx: discord.ApplicationContext, raw: discord.Option(discord.Attachment)
    ):
        new = MarkovChain.loadb(await raw.read())
        self.markov.merge(new)
        await ctx.respond("Imported", ephemeral=True)
        await self.update_words()

    @require_owner
    @commands.slash_command()
    async def scan_history(self, ctx: discord.ApplicationContext):
        await ctx.defer(ephemeral=True)
        async for msg in ctx.history(limit=None):
            if msg.author.id != self.bot.application_id:
                self.markov.learn(msg.content)
        await ctx.respond("> Bingus Learned!", ephemeral=True)
        await self.update_words()

    @commands.slash_command()
    async def markov(
        self, ctx: discord.ApplicationContext, prompt: discord.Option(str)
    ):
        print("Bingus is responding!")
        response = self.markov.respond(prompt)
        if response is not None and len(response) != 0:
            await ctx.respond(f"{prompt} {response[:1000]}")
        else:
            await ctx.respond("> Bingus couldn't think of what to say!")
        print("Bingus Responded!")

    @commands.slash_command()
    async def weights(
        self, ctx: discord.ApplicationContext, token: discord.Option(str)
    ):
        edges = self.markov.get_edges(token)
        if edges is None:
            await ctx.respond("> Bingus doesn't know that word!")
        else:
            head = f"Weights for **{token}**"
            msg = "\n".join([f"{str(k)}: {v}" for k, v in edges.items()])
            if len(msg) > 1750:
                fd = io.BytesIO(msg.encode())
                await ctx.respond(head, file=discord.File(fd, filename="weights.txt"))
            else:
                await ctx.respond(f"{head}:\n{msg}")

    @require_owner
    @commands.slash_command()
    async def ocr(
        self, ctx: discord.ApplicationContext, file: discord.Option(discord.Attachment)
    ):
        raw = await file.read()
        try:
            image = PIL.Image.open(io.BytesIO(raw))
            text = pytesseract.image_to_string(image)
            self.markov.learn(text)
            await ctx.respond("> Bingus learned something from image!", ephemeral=True)
            await self.update_words()
        except PIL.UnidentifiedImageError:
            await ctx.respond(
                "> Bingus only understands image files!", ephemeral=True
            )

    @require_owner
    @commands.slash_command()
    async def study(
        self, ctx: discord.ApplicationContext, file: discord.Option(discord.Attachment)
    ):
        raw = await file.read()
        try:
            text = raw.decode()
            self.markov.learn(text)
            await ctx.respond("> Bingus learned from file!", ephemeral=True)
            await self.update_words()
        except UnicodeDecodeError:
            await ctx.respond(
                "> Bingus only understands UTF-8 text files!", ephemeral=True
            )

    @require_owner
    @commands.slash_command()
    async def forget(
        self, ctx: discord.ApplicationContext):
            self.markov.forget()
            await ctx.respond("> Bingus forgot everything!", ephemeral=True)
            await self.update_words()

    @commands.Cog.listener()
    async def on_ready(self):
        await self.update_words()

    @commands.Cog.listener()
    async def on_message(self, msg: Message):
        if msg.flags.ephemeral or msg.channel.type == discord.ChannelType.private:
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
