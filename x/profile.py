"""Profile AoC solutions with samply."""

import os
import subprocess
import sys

import click

from .inputs.decrypt import decrypt_inputs
from .paths import ROOT_DIR, input_path
from .types import Day, Part, Year, validate_int


def profile_solution(year: Year, day: Day, part: Part) -> int:
    """
    Profile a specific AoC solution using samply.

    Args:
        year: The year of the puzzle
        day: The day of the puzzle
        part: The part of the puzzle (1 or 2)

    Returns:
        The exit code from samply
    """
    # Build with profiling profile
    build_result = subprocess.run(
        ["cargo", "build", "--profile=profiling", "--package", "aoc"],
        stdin=sys.stdin,
        stdout=sys.stdout,
        stderr=sys.stderr,
        env={**os.environ, "RUSTFLAGS": "-C target-cpu=native"},
    )

    if build_result.returncode != 0:
        return build_result.returncode

    # Run with samply
    binary_path = ROOT_DIR / "target" / "profiling" / "aoc"
    args = [
        "samply",
        "record",
        str(binary_path),
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
    ).returncode


def register(cli: click.Group) -> None:
    """Register the profile command with the CLI."""

    @cli.command()
    @click.argument("year", type=int, callback=validate_int(Year))
    @click.argument("day", type=int, callback=validate_int(Day))
    @click.argument("part", type=int, callback=validate_int(Part))
    def profile(year: Year, day: Day, part: Part) -> int:
        """
        Profile an AoC solution with samply.

        Requires YEAR, DAY, and PART arguments.
        """
        decrypt_inputs()
        return profile_solution(year, day, part)
