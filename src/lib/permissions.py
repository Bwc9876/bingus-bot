import discord

class NotOwnerError(discord.ApplicationCommandError):

    def __init__(self) -> None:
        super().__init__("You are not allowed to run this command")


def _check_owners(ctx: discord.ApplicationContext):
    if ctx.author.id not in ctx.bot.bingus_owners:
        raise NotOwnerError()
    else:
        return True
    

def require_owner(cmd: discord.SlashCommand):
    cmd.checks.append(_check_owners)
    return cmd


__all__ = (require_owner, NotOwnerError)
