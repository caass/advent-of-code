"""CLI tool for managing Advent of Code solutions."""

import click

from . import completion, inputs, profile, run, test
from .types import Context


@click.group()
def cli() -> None:
    """Utility CLI for interacting with Advent of Code solutions."""
    pass


# Register all commands
test.register(cli)
completion.register(cli)
inputs.register(cli)
run.register(cli)
profile.register(cli)


def main(args: list[str]) -> None:
    """Entry point for the CLI."""
    extra_args: list[str] = []

    if "--" in args:
        extra_args = args[args.index("--") + 1 :]
        args = args[: args.index("--")]

    cli(args[1:], obj=Context(extra_args=extra_args))
