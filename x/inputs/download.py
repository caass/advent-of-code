"""Download AoC puzzle inputs from adventofcode.com."""

import time
from datetime import datetime

import click
import httpx
from yaspin import yaspin
from yaspin.spinners import Spinners

from ..paths import INPUTS_DIR, input_path
from ..types import Day, Year
from .browser import Browser, get_session_cookie
from .encrypt import encrypt_inputs
from .lockfile import Lockfile, hash_content

# Minimum delay between requests (in seconds) to be nice to the AoC server
REQUEST_DELAY = 0.5


class _RateLimitedTransport(httpx.HTTPTransport):
    """
    HTTP transport that enforces a minimum delay between requests.

    This prevents hammering the AoC server and getting rate-limited or banned.
    """

    def __init__(self, delay: float = REQUEST_DELAY, **kwargs):
        super().__init__(**kwargs)
        self._delay = delay
        self._last_request: float = 0

    def handle_request(self, request: httpx.Request) -> httpx.Response:
        # Enforce minimum delay since last request
        elapsed = time.monotonic() - self._last_request
        if elapsed < self._delay:
            time.sleep(self._delay - elapsed)

        self._last_request = time.monotonic()
        return super().handle_request(request)


class AocClient(httpx.Client):
    """
    HTTP client for downloading Advent of Code puzzle inputs.

    Features:
    - Rate limiting to avoid hammering the AoC server
    - Session cookie authentication (auto-fetched from browser if not provided)
    - Methods for fetching inputs by year/day
    """

    def __init__(
        self,
        lockfile: Lockfile | None = None,
        session_cookie: str | None = None,
        browser: Browser | None = None,
        **kwargs,
    ):
        if session_cookie is None:
            session_cookie = get_session_cookie(browser)

        super().__init__(
            transport=_RateLimitedTransport(),
            cookies={"session": session_cookie},
            headers={"User-Agent": "github.com/caass/advent-of-code x.py"},
            **kwargs,
        )
        self._lockfile = lockfile if lockfile is not None else Lockfile()

    def input(self, year: Year, day: Day) -> str:
        """
        Fetch a single puzzle input from adventofcode.com.

        Args:
            year: The year of the puzzle
            day: The day of the puzzle

        Returns:
            The puzzle input as a string

        Raises:
            click.ClickException: If the request fails
        """
        url = f"https://adventofcode.com/{year}/day/{day}/input"
        try:
            response = self.get(url)
            response.raise_for_status()
            return response.text
        except httpx.HTTPStatusError as e:
            if e.response.status_code == 404:
                raise click.ClickException(
                    f"Puzzle input not found for {year} day {day}. "
                    "The puzzle may not be released yet."
                )
            elif e.response.status_code == 400:
                raise click.ClickException(
                    f"Bad request for {year} day {day}. "
                    "Your session cookie may be invalid or expired."
                )
            else:
                raise click.ClickException(
                    f"HTTP error {e.response.status_code} fetching {url}: {e}"
                )
        except httpx.RequestError as e:
            raise click.ClickException(
                f"Network error fetching {year} day {day}: {e}. "
                "Check your internet connection and try again."
            )

    def year(self, year: Year, days: list[Day], force: bool = False) -> None:
        """
        Download inputs for a specific year with progress spinner.

        Args:
            year: The year to download inputs for
            days: List of days to download
            force: If True, re-download even if cached
        """
        year_path = INPUTS_DIR / str(year)
        year_path.mkdir(exist_ok=True)

        total_days = len(days)
        downloaded = 0

        with yaspin(
            Spinners.dots, text=f"{year} Downloading [00/{total_days:02d}]"
        ) as sp:
            current = 0
            for day in days:
                if not force and not self._lockfile.needs_download(year, day):
                    current += 1
                    sp.text = f"{year} Downloading [{current:02d}/{total_days:02d}]"
                    continue

                sp.text = f"{year} Downloading [{current:02d}/{total_days:02d}]"
                content = self.input(year, day)
                input_path(year, day).write_text(content)

                # Update lockfile with new hash
                self._lockfile.set(year, day, hash_content(content))
                current += 1
                downloaded += 1
                sp.text = f"{year} Downloading [{current:02d}/{total_days:02d}]"

            cached_suffix = " (cached)" if downloaded == 0 else ""
            sp.text = f"{year} Done!{cached_suffix}"
            sp.ok("✔")


def ensure_input(year: Year, day: Day, browser: Browser | None = None) -> bool:
    """
    Ensure a specific input is downloaded.

    Downloads the input if it's missing from the lockfile, then encrypts inputs.

    Args:
        year: The year of the puzzle
        day: The day of the puzzle
        browser: Specific browser to get cookies from. If None, auto-detects.

    Returns:
        True if the input was downloaded, False if it was already cached.
    """
    lockfile = Lockfile()

    if not lockfile.needs_download(year, day):
        return False

    # Ensure inputs directory exists
    year_path = INPUTS_DIR / str(year)
    year_path.mkdir(parents=True, exist_ok=True)

    with yaspin(Spinners.dots, text=f"Downloading {year} day {day}...") as sp:
        with AocClient(lockfile=lockfile, browser=browser) as client:
            content = client.input(year, day)
            input_path(year, day).write_text(content)

            # Update lockfile with new hash
            lockfile.set(year, day, hash_content(content))
            lockfile.save()

        sp.text = f"Downloaded {year} day {day}"
        sp.ok("✔")

    # Auto-encrypt after download
    encrypt_inputs()

    return True


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
    lockfile = Lockfile()

    # Ensure inputs directory exists
    INPUTS_DIR.mkdir(parents=True, exist_ok=True)

    now = datetime.now()
    current_year = Year(now.year)
    current_month = now.month
    today = now.day

    with AocClient(lockfile=lockfile, browser=browser) as client:
        # Download all past years (all days for each year)
        for year in Year.before(current_year):
            client.year(year, list(year.days()), force=force)

        # Download current year if it's December (only up to available days)
        if current_month == 12:
            # Cap at the number of days available for this year's event
            max_day = min(today, current_year.num_days())
            client.year(current_year, list(Day.up_to(max_day)), force=force)

    # Save lockfile
    lockfile.save()

    # Auto-encrypt after download
    encrypt_inputs()
