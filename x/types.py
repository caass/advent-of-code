"""Shared types and utilities for the x.py CLI tool."""

import enum
import typing as t
from dataclasses import dataclass, field

import click
from junitparser import TestSuite


class Year(enum.IntEnum):
    """
    An Advent of Code year.

    AoC started in 2015 and runs annually through December.
    """

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
    TwentyTwentyFive = 2025

    def __str__(self) -> str:
        return str(self.value)

    def num_days(self) -> int:
        """Return the number of days for this year's AoC event."""
        if self >= Year.TwentyTwentyFive:
            return 12
        return 25

    def days(self) -> t.Iterator["Day"]:
        """Iterate over all days for this year's AoC event."""
        return Day.up_to(self.num_days())

    @classmethod
    def up_to(cls, year: "Year") -> t.Iterator["Year"]:
        """Iterate over all years from 2015 up to and including the given year."""
        for y in cls:
            if y <= year:
                yield y

    @classmethod
    def before(cls, year: "Year") -> t.Iterator["Year"]:
        """Iterate over all years from 2015 up to but not including the given year."""
        for y in cls:
            if y < year:
                yield y

    @staticmethod
    def from_test_suite(suite: TestSuite) -> "Year":
        """Extract the year from a JUnit test suite name like 'aoc-2024::integration'."""
        return Year(int(suite.name.removeprefix("aoc-").removesuffix("::integration")))


class Day(enum.IntEnum):
    """
    An Advent of Code day (1-25).

    Each December, AoC releases one puzzle per day from December 1st through 25th.
    Beginning in 2025, AoC releases one puzzle per day from December 1st through 12th.
    """

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

    def __str__(self) -> str:
        return str(self.value)

    @classmethod
    def up_to(cls, n: int) -> t.Iterator["Day"]:
        """Iterate over days from 1 up to and including n."""
        for day in cls:
            if day <= n:
                yield day


class Part(enum.IntEnum):
    """
    A puzzle part (1 or 2).

    Each AoC day has two parts; part 2 unlocks after solving part 1.
    Day 25 only has one part.
    """

    Part1 = 1
    Part2 = 2

    def __str__(self) -> str:
        return str(self.value)


class ProblemState(enum.Enum):
    """Tracks completion status of an AoC puzzle."""

    Unsolved = enum.auto()
    """Neither part has been solved."""

    PartiallySolved = enum.auto()
    """Only part 1 has been solved."""

    Solved = enum.auto()
    """Both parts have been solved (or just part 1 for day 25)."""


@dataclass
class Context:
    """
    CLI context object passed between commands.

    Holds extra arguments passed after '--' which are forwarded to underlying tools.
    """

    extra_args: list[str] = field(default_factory=list[str])


def validate_int[T](
    ty: t.Callable[[int], T],
) -> t.Callable[[click.Context, click.Parameter, int | None], T | None]:
    """
    Create a Click callback that validates and converts an integer to a typed enum.

    Used with @click.argument to convert raw integers to Year, Day, or Part enums
    while providing helpful error messages for invalid values.
    """

    def _validate_impl(ctx: click.Context, param: click.Parameter, value: int | None):
        if value is None:
            return None

        try:
            return ty(value)
        except ValueError as err:
            raise click.BadParameter(str(err), ctx=ctx, param=param)

    return _validate_impl
