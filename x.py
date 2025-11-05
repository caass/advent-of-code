#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.13"
# dependencies = [
#    "junitparser>=4.0.2",
#    "click>=8.3.0",
#    "py-markdown-table>=1.3.0",
# ]
# ///

import os
import click
import enum
from datetime import datetime
from junitparser import JUnitXml
from py_markdown_table.markdown_table import markdown_table


class Year(enum.IntEnum):
    TwentyFifteen = 2015
    TwentySixteen = 2016
    TwentySeventeen = 2017
    TwentyEighteen = 2018
    TwentyNineteen = 2019
    TwentyTwenty = 2020
    TwentyTwentyOne = 2021
    TwentyTwentyTwo = 2022
    TwentyTwentyThree = 2023
    TwentyTwentyFour = 2024


class Day(enum.IntEnum):
    Day01 = 1
    Day02 = 2
    Day03 = 3
    Day04 = 4
    Day05 = 5
    Day06 = 6
    Day07 = 7
    Day08 = 8
    Day09 = 9
    Day10 = 10
    Day11 = 11
    Day12 = 12
    Day13 = 13
    Day14 = 14
    Day15 = 15
    Day16 = 16
    Day17 = 17
    Day18 = 18
    Day19 = 19
    Day20 = 20
    Day21 = 21
    Day22 = 22
    Day23 = 23
    Day24 = 24
    Day25 = 25


class ProblemState(enum.Enum):
    Unsolved = enum.auto()
    PartiallySolved = enum.auto()
    Solved = enum.auto()


class Completion(dict[Year, dict[Day, ProblemState]]):
    def __init__(self):
        super().__init__()

        for year in Year:
            self[year] = {}
            for day in Day:
                if year == 2025 and day == 13:
                    break  # only 12 days starting in 2025

                self[year][day] = ProblemState.Unsolved

        junit_file = os.path.join(
            os.path.dirname(os.path.realpath(__file__)),
            "target",
            "nextest",
            "ci",
            "junit.xml",
        )

        xml = JUnitXml.fromfile(junit_file)
        for suite in xml:
            if not suite.name.endswith("::integration"):
                continue

            year = Year(
                int(suite.name.removeprefix("aoc-").removesuffix("::integration"))
            )

            for case in suite:
                name_parts = case.name.split("::")
                if not len(name_parts) == 2:
                    raise IndexError(
                        f"expected test in {suite.name} to be named day::part, found {case.name}"
                    )

                [daystr, partstr] = name_parts
                day = Day(int(daystr.removeprefix("day")))
                part = int(partstr.removeprefix("part"))

                if part == 1 and self[year][day] == ProblemState.Unsolved:
                    self[year][day] = ProblemState.PartiallySolved
                elif part == 2:
                    self[year][day] = ProblemState.Solved

    def table(self) -> markdown_table:
        rows: list[dict[str, int | str]] = []

        for year, days in self.items():
            available_stars = len(days) * 2
            attained_stars = 0

            for state in days.values():
                if state == ProblemState.PartiallySolved:
                    attained_stars += 1
                elif state == ProblemState.Solved:
                    attained_stars += 2

            # bump for the last day, which has only one part
            if attained_stars == available_stars - 1:
                attained_stars += 1

            rows.append(
                {
                    "Year": year,
                    "Earned ⭐️": attained_stars,
                    "Possible ⭐️": available_stars,
                    "Complete": str(int(100 * attained_stars / available_stars)) + "%",
                }
            )

        return markdown_table(rows)


@click.group()
def x():
    return


@x.command()
def completion():
    completion = Completion()
    readme_path = os.path.join(os.path.dirname(os.path.realpath(__file__)), "README.md")

    with open(readme_path, "r") as readme:
        divider = "<!-- INSERT COMPLETION TABLE -->"
        readme_str = readme.read()
        readme_parts = readme_str.split(divider)
        if len(readme_parts) != 3:
            raise IndexError("expected readme to have exactly two completion comments")

        [prefix, _, suffix] = readme_parts
        readme_str = (
            prefix
            + divider
            + "\n"
            + completion.table()
            .set_params(quote=False, row_sep="markdown")
            .get_markdown()
            + "\n"
            + divider
            + suffix
        )

    with open(readme_path, "w") as readme:
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
