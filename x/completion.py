"""Generate and update the completion table in README.md."""

import click
from junitparser import JUnitXml
from py_markdown_table.markdown_table import markdown_table

from .paths import README, junit_path
from .test import run_tests
from .types import Day, Part, ProblemState, Year


class Completion(dict[Year, dict[Day, ProblemState]]):
    """
    Tracks puzzle completion status across all years and days.

    Parses JUnit XML test results to determine which puzzles have been solved.
    """

    def __init__(self, junit: JUnitXml):
        super().__init__(
            {year: {day: ProblemState.Unsolved for day in year.days()} for year in Year}
        )

        for suite in junit:
            year = Year.from_test_suite(suite)
            if year is None:
                continue

            last_day = Day(year.num_days())

            for case in suite:
                name_parts = case.name.split("::")
                if len(name_parts) != 2:
                    raise click.ClickException(
                        f"Expected test in {suite.name} to be named day::part, found {case.name}"
                    )

                [daystr, partstr] = name_parts
                day = Day(int(daystr.removeprefix("day")))
                part = Part(int(partstr.removeprefix("part")))

                if part == 1 and self[year][day] == ProblemState.Unsolved:
                    # Last day only has part 1, so completing it means fully solved
                    if day == last_day:
                        self[year][day] = ProblemState.Solved
                    else:
                        self[year][day] = ProblemState.PartiallySolved
                elif part == 2:
                    self[year][day] = ProblemState.Solved

    def table(self) -> markdown_table:
        """Generate a markdown table showing completion stats per year."""
        rows: list[dict[str, int | str]] = []

        for year, days in self.items():
            # Each day awards 2 stars, except the last day which only has 1 part.
            # Completing all puzzles awards a bonus star, so total = days * 2.
            num_days = year.num_days()
            available_stars = num_days * 2
            attained_stars = 0

            last_day = Day(num_days)
            all_solved = True
            for day, state in days.items():
                if state == ProblemState.PartiallySolved:
                    attained_stars += 1
                    all_solved = False
                elif state == ProblemState.Solved:
                    # Last day only awards 1 star for "Solved" since it has no part 2
                    attained_stars += 1 if day == last_day else 2
                else:
                    all_solved = False

            # Bonus star for completing all puzzles
            if all_solved:
                attained_stars += 1

            rows.append(
                {
                    "Year": year,
                    "Earned ⭐️": attained_stars,
                    "Possible ⭐️": available_stars,
                    "Complete": f"{int(100 * attained_stars / available_stars)}%",
                }
            )

        return markdown_table(rows)


def update_completion_table() -> None:
    """
    Run all integration tests and update the README completion table.

    Runs tests with the completion profile to generate JUnit XML output, then
    parses the results to build a completion table showing stars earned per year.
    """
    run_tests(extra_args=["--profile=completion"])

    comp = Completion(JUnitXml.fromfile(str(junit_path("completion"))))

    divider = "<!-- INSERT COMPLETION TABLE -->"
    readme_str = README.read_text()
    readme_parts = readme_str.split(divider)
    if len(readme_parts) != 3:
        raise click.ClickException(
            f"Expected README to have exactly two '{divider}' comments, found {len(readme_parts) - 1}"
        )

    [prefix, _, suffix] = readme_parts
    readme_str = (
        prefix
        + divider
        + "\n"
        + comp.table().set_params(quote=False, row_sep="markdown").get_markdown()
        + "\n"
        + divider
        + suffix
    )

    README.write_text(readme_str)


def register(cli: click.Group) -> None:
    """Register the completion command with the CLI."""

    @cli.command()
    def completion() -> None:
        """Update the completion table in README.md based on passing tests."""
        update_completion_table()
