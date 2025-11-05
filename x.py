#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.13"
# dependencies = [
#    "junitparser>=4.0.2",
#    "click>=8.3.0",
#    "browsercookie>=0.8.2",
#    "py-markdown-table>=1.3.0"
# ]
# ///

import os
import click
import enum
from datetime import datetime
from junitparser import JUnitXml
from py_markdown_table.markdown_table import markdown_table


@click.group()
def x():
    return


class SolveState(enum.Enum):
    Unsolved = enum.auto()
    PartiallySolved = enum.auto()
    Solved = enum.auto()


@x.command()
def completion():
    root_dir = os.path.dirname(os.path.realpath(__file__))
    target_dir = os.path.join(root_dir, "target")

    if not os.path.isdir(target_dir):
        raise FileNotFoundError(target_dir)

    inputs_dir = os.path.join(target_dir, "inputs")
    if not os.path.isdir(inputs_dir):
        raise FileNotFoundError(inputs_dir)

    now = datetime.now()
    latest_year = now.year
    if now.month == 12:
        latest_year += 1

    years: dict[int, dict[int, SolveState]] = {}

    for year in range(2015, latest_year):
        years[year] = {}

        year_inputs_dir = os.path.join(inputs_dir, str(year))
        if not os.path.isdir(year_inputs_dir):
            raise FileNotFoundError(year_inputs_dir)

        for file in os.listdir(year_inputs_dir):
            years[year][int(file)] = SolveState.Unsolved

    junit_file = os.path.join(target_dir, "nextest", "ci", "junit.xml")
    xml = JUnitXml.fromfile(junit_file)
    for suite in xml:
        if not suite.name.endswith("::integration"):
            continue

        year = int(suite.name.removeprefix("aoc-").removesuffix("::integration"))

        for case in suite:
            name_parts = case.name.split("::")
            if not len(name_parts) == 2:
                raise IndexError(
                    f"expected test in {suite.name} to be named day::part, found {case.name}"
                )

            [daystr, partstr] = name_parts
            day = int(daystr.removeprefix("day"))
            part = int(partstr.removeprefix("part"))

            if part == 1 and years[year][day] == SolveState.Unsolved:
                years[year][day] = SolveState.PartiallySolved
            elif part == 2:
                years[year][day] = SolveState.Solved

    rows: list[dict[str, int]] = []
    for year, days in years.items():
        available_stars = len(days) * 2
        attained_stars = 0
        for state in days.values():
            if state == SolveState.PartiallySolved:
                attained_stars += 1
            elif state == SolveState.Solved:
                attained_stars += 2

        # bump for the last day, which has only one part
        if attained_stars == available_stars - 1:
            attained_stars += 1

        rows.append(
            {"Year": year, "Earned ⭐️": attained_stars, "Possible ⭐️": available_stars}
        )

    with open(os.path.join(root_dir, "README.md"), "r") as readme:
        divider = "<!-- INSERT COMPLETION TABLE -->"
        readme_str: str = readme.read()
        readme_parts = readme_str.split(divider)
        if len(readme_parts) != 3:
            raise IndexError("expected readme to have exactly two completion comments")

        [prefix, _, suffix] = readme_parts
        readme_str = (
            prefix
            + divider
            + markdown_table(rows).set_params(quote=False).get_markdown()
            + "\n"
            + divider
            + suffix
        )

    with open(os.path.join(root_dir, "README.md"), "w") as readme:
        readme.write(readme_str)


@x.group()
def inputs():
    return


@inputs.command()
def download():
    return


@inputs.command()
def encrypt():
    return


@inputs.command()
def decrypt():
    return


if __name__ == "__main__":
    x()
