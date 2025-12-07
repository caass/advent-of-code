"""Manage AoC puzzle inputs: download, encrypt, and decrypt."""

import click

from .browser import Browser, validate_browser
from .decrypt import decrypt_inputs
from .download import download_inputs
from .encrypt import encrypt_inputs


def register(cli: click.Group) -> None:
    """Register the inputs command group with the CLI."""

    @cli.group()
    def inputs() -> None:
        """Manage puzzle inputs (download, encrypt, decrypt)."""
        pass

    @inputs.command()
    @click.option(
        "--browser",
        "-b",
        type=click.Choice([b.value for b in Browser], case_sensitive=False),
        callback=validate_browser,
        help="Browser to get session cookie from. Auto-detects if not specified.",
    )
    @click.option(
        "--force",
        "-f",
        is_flag=True,
        help="Re-download all inputs, ignoring the lockfile cache.",
    )
    def download(browser: Browser | None, force: bool) -> None:
        """Download puzzle inputs from adventofcode.com."""
        download_inputs(browser, force)

    @inputs.command()
    def encrypt() -> None:
        """Encrypt puzzle inputs for storage in the repository."""
        encrypt_inputs()

    @inputs.command()
    def decrypt() -> None:
        """Decrypt puzzle inputs for running solutions."""
        decrypt_inputs()
