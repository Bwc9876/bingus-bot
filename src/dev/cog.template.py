
import discord
from discord.ext import commands
from discord.message import Message

class __CName__(commands.Cog):

    # Setup any state for the cog here, this will
    # exist for the run of the program
    # If you need to change the parameters for __init__
    # make sure the [setup] function below this class.
    def __init__(self, bot) -> None:
        self.bot = bot

    # Example of a slash command that will be loaded
    # with this cog
    # @commands.slash_command()
    # async def ping(self, ctx: discord.ApplicationContext):
    #     await ctx.respond("pong!")

    # Example of listening to all messages
    # ever sent in any server the bot is in
    # while active
    # @commands.Cog.listener()
    # async def on_message(self, msg: Message):
    #     pass

    # See the PyCord docs for more info and guides

def setup(bot):
    bot.add_cog(__CName__(bot))
