"""Run benchmarks for AoC solutions using divan."""

import subprocess
import sys

import click

from .inputs.decrypt import decrypt_inputs
from .inputs.download import ensure_input
from .types import Day, Part, Year, validate_int


def run_benchmarks(
    year: Year | None = None,
    day: Day | None = None,
    part: Part | None = None,
    extra_args: list[str] | None = None,
) -> subprocess.CompletedProcess[bytes]:
    """
    Run benchmarks using cargo bench with divan.

    Args:
        year: Filter to a specific year's benchmarks
        day: Filter to a specific day's benchmarks (requires year)
        part: Filter to a specific part's benchmarks (requires year and day)
        extra_args: Additional arguments to pass to divan

    Returns:
        The completed process from cargo bench
    """
    args = [
        "cargo",
        "bench",
        "--bench=aoc",
    ]

    # Build the divan filter
    # Divan uses module path matching: y2015::d1::part1
    divan_filter: str | None = None

    if year is not None:
        divan_filter = f"y{year}::"
        if day is not None:
            divan_filter += f"d{day}::"
            if part is not None:
                divan_filter += f"part{part}"

    # Add -- separator for divan args
    args.append("--")

    # Force color output (useful when piping to tee in CI)
    args.append("--color=always")

    if divan_filter is not None:
        args.append(divan_filter)

    if extra_args:
        args.extend(extra_args)

    click.echo(" ".join(args))
    return subprocess.run(
        args=args,
        stdin=sys.stdin,
        stdout=sys.stdout,
        stderr=sys.stderr,
    )


def register(cli: click.Group) -> None:
    """Register the bench command with the CLI."""

    @cli.command()
    @click.argument("year", type=int, required=False, callback=validate_int(Year))
    @click.argument("day", type=int, required=False, callback=validate_int(Day))
    @click.argument("part", type=int, required=False, callback=validate_int(Part))
    @click.pass_context
    def bench(
        ctx: click.Context,
        year: Year | None = None,
        day: Day | None = None,
        part: Part | None = None,
    ) -> int:
        """
        Run benchmarks for AoC solutions.

        Optionally filter by YEAR, DAY, and PART.

        Extra arguments after -- are passed to divan. Examples:

        \b
          ./x.py bench                      # Run all benchmarks
          ./x.py bench 2015                 # Run 2015 benchmarks
          ./x.py bench 2015 1               # Run 2015 day 1 benchmarks
          ./x.py bench 2015 1 2             # Run 2015 day 1 part 2 benchmark
          ./x.py bench -- --sample-count 10 # Run all with 10 samples
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

        # Auto-download input if benchmarking a specific day
        if year is not None and day is not None:
            ensure_input(year, day)

        decrypt_inputs()

        extra_args = getattr(ctx.obj, "extra_args", []) if ctx.obj else []
        ctx.exit(
            run_benchmarks(
                year=year, day=day, part=part, extra_args=extra_args
            ).returncode
        )
