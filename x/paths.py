"""Common path constants for the x.py CLI tool."""

from pathlib import Path

from .types import Day, Year

# Project root directory (where x.py lives)
ROOT_DIR = Path(__file__).parent.parent

# Decrypted inputs directory
INPUTS_DIR = ROOT_DIR / "target" / "inputs"

# Encrypted inputs archive
INPUTS_ARCHIVE = ROOT_DIR / "inputs.gz.age"

# README file
README = ROOT_DIR / "README.md"

# Lockfile for tracking downloaded inputs
LOCKFILE = ROOT_DIR / "aoc.lock"


def input_path(year: Year, day: Day) -> Path:
    """Get the path to a specific puzzle input file."""
    return INPUTS_DIR / str(year) / f"{day:02d}"


def junit_path(profile: str = "ci") -> Path:
    """Get the path to the JUnit XML output from nextest."""
    return ROOT_DIR / "target" / "nextest" / profile / "junit.xml"
