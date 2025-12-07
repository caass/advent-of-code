"""Run AoC solutions."""

import os
import subprocess
import sys

import click

from .inputs.decrypt import decrypt_inputs
from .paths import input_path
from .types import Day, Part, Year, validate_int


def run_solution(
    year: Year, day: Day, part: Part
) -> subprocess.CompletedProcess[bytes]:
    """
    Run a specific AoC solution.

    Args:
        year: The year of the puzzle
        day: The day of the puzzle
        part: The part of the puzzle (1 or 2)

    Returns:
        The exit code from cargo run
    """
    args = [
        "cargo",
        "run",
        "--release",
        "--package",
        "aoc",
        "--",
        str(year),
        str(day),
        str(part),
        str(input_path(year, day)),
    ]

    return subprocess.run(
        args=args,
        stdin=sys.stdin,
        stdout=sys.stdout,
        stderr=sys.stderr,
        env={**os.environ, "RUSTFLAGS": "-C target-cpu=native"},
    )


def register(cli: click.Group) -> None:
    """Register the run command with the CLI."""

    @cli.command()
    @click.argument("year", type=int, callback=validate_int(Year))
    @click.argument("day", type=int, callback=validate_int(Day))
    @click.argument("part", type=int, callback=validate_int(Part))
    @click.pass_context
    def run(ctx: click.Context, year: Year, day: Day, part: Part) -> None:
        """
        Run an AoC solution.

        Requires YEAR, DAY, and PART arguments.
        """
        decrypt_inputs()
        ctx.exit(run_solution(year, day, part).returncode)
