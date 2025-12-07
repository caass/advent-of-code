"""Run tests for AoC solutions using cargo nextest."""

import subprocess
import sys

import click

from .types import Day, Part, Year, validate_int


def run_tests(
    year: Year | None = None,
    day: Day | None = None,
    part: Part | None = None,
    extra_args: list[str] | None = None,
) -> int:
    """
    Run tests using cargo nextest.

    Args:
        year: Filter to a specific year's tests
        day: Filter to a specific day's tests (requires year)
        part: Filter to a specific part's tests (requires year and day)
        extra_args: Additional arguments to pass to cargo nextest

    Returns:
        The exit code from cargo nextest
    """
    args = [
        "cargo",
        "nextest",
        "run",
        "--no-tests=fail",
        "--cargo-profile=fast-test",
    ]

    if extra_args:
        args.extend(extra_args)

    if year is None:
        args.append("--workspace")
    else:
        args.append(f"--package=aoc-{year}")

        if day is not None:
            # Build the test filter: "day01" or "day01::part1"
            test_filter = f"day{day:02d}"
            if part is not None:
                test_filter += f"::part{part}"
            args.extend(["--", test_filter])

    click.echo(" ".join(args))
    return subprocess.run(
        args=args,
        stdin=sys.stdin,
        stdout=sys.stdout,
        stderr=sys.stderr,
    ).returncode


def register(cli: click.Group) -> None:
    """Register the test command with the CLI."""

    @cli.command()
    @click.argument("year", type=int, required=False, callback=validate_int(Year))
    @click.argument("day", type=int, required=False, callback=validate_int(Day))
    @click.argument("part", type=int, required=False, callback=validate_int(Part))
    @click.pass_context
    def test(
        ctx: click.Context,
        year: Year | None = None,
        day: Day | None = None,
        part: Part | None = None,
    ) -> int:
        """
        Run tests for AoC solutions.

        Optionally filter by YEAR, DAY, and PART.
        """
        if year is None and (day is not None or part is not None):
            raise click.UsageError(
                ctx=ctx,
                message="cannot specify day or part without specifying year",
            )
        if day is None and part is not None:
            raise click.UsageError(
                ctx=ctx,
                message="cannot specify part without specifying year and day",
            )

        extra_args = getattr(ctx.obj, "extra_args", []) if ctx.obj else []
        return run_tests(year=year, day=day, part=part, extra_args=extra_args)
