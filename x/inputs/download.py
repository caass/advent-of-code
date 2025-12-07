"""Download AoC puzzle inputs from adventofcode.com."""

import time
import typing as t
from datetime import datetime

import click
import httpx

from ..paths import INPUTS_DIR, input_path
from ..types import Day, Year
from .browser import Browser, get_session_cookie
from .lockfile import Lockfile, hash_content


def download_inputs(browser: Browser | None = None, force: bool = False) -> None:
    """
    Download puzzle inputs from adventofcode.com.

    Retrieves the session cookie from your browser automatically.
    Downloads all inputs for past years plus the current year (if it's December).

    Uses a lockfile (aoc.lock) to track which inputs have been downloaded and
    their SHA-256 hashes. Inputs that already exist with matching hashes are
    skipped unless --force is used.

    Args:
        browser: Specific browser to get cookies from. If None, auto-detects.
        force: If True, re-download all inputs regardless of lockfile state.
    """
    session_cookie = get_session_cookie(browser)
    lockfile = Lockfile()

    # Ensure inputs directory exists
    INPUTS_DIR.mkdir(parents=True, exist_ok=True)

    now = datetime.now()
    current_year = Year(now.year)
    current_month = now.month
    today = now.day

    client = httpx.Client(
        cookies={"session": session_cookie},
        headers={"User-Agent": "github.com/caass/advent-of-code x.py"},
    )

    def fetch_input(year: Year, day: Day) -> str:
        """Fetch a single puzzle input from adventofcode.com."""
        time.sleep(0.5)  # Be nice to the server
        response = client.get(f"https://adventofcode.com/{year}/day/{day}/input")
        response.raise_for_status()
        return response.text

    def download_year(year: Year, days: t.Iterator[Day]) -> None:
        """Download inputs for a specific year."""
        year_path = INPUTS_DIR / str(year)
        year_path.mkdir(exist_ok=True)

        click.echo(f"{year}: ", nl=False)
        downloaded = 0
        skipped = 0

        for day in days:
            if not force and not lockfile.needs_download(year, day):
                skipped += 1
                continue

            click.echo(f"{day}, ", nl=False)
            content = fetch_input(year, day)
            input_path(year, day).write_text(content)

            # Update lockfile with new hash
            lockfile.set(year, day, hash_content(content))
            downloaded += 1

        if downloaded == 0 and skipped > 0:
            click.echo(f"all {skipped} inputs cached")
        elif skipped > 0:
            click.echo(f"Done! ({skipped} cached)")
        else:
            click.echo("Done!")

    click.echo("Downloading inputs...")

    # Download all past years (all days for each year)
    for year in Year.before(current_year):
        download_year(year, year.days())

    # Download current year if it's December (only up to today)
    if current_month == 12:
        download_year(current_year, Day.up_to(today))

    # Save lockfile
    lockfile.save()
    client.close()
