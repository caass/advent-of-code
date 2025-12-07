"""Encrypt AoC puzzle inputs for storage in the repository."""

import gzip
import io
import os
import tarfile

import click
import pyrage

from ..paths import INPUTS_ARCHIVE, INPUTS_DIR


def encrypt_inputs() -> None:
    """
    Encrypt puzzle inputs from target/inputs/ to inputs.gz.age.

    Requires the AOC_INPUTS_PUBKEY environment variable to be set to the
    age public key.
    """
    pubkey = os.environ.get("AOC_INPUTS_PUBKEY")
    if not pubkey:
        raise click.ClickException(
            "Need AOC_INPUTS_PUBKEY to be set to encrypt puzzle inputs."
        )

    if not INPUTS_DIR.exists():
        raise click.ClickException(f"Inputs directory not found: {INPUTS_DIR}")

    # Parse the recipient from the public key
    recipient = pyrage.x25519.Recipient.from_str(pubkey.strip())

    # Create tar archive in memory
    tar_buffer = io.BytesIO()
    with tarfile.open(fileobj=tar_buffer, mode="w") as tar:
        tar.add(INPUTS_DIR, arcname="./target/inputs")
    tar_data = tar_buffer.getvalue()

    # Compress with gzip
    gz_buffer = io.BytesIO()
    with gzip.GzipFile(fileobj=gz_buffer, mode="wb") as gz:
        gz.write(tar_data)
    compressed_data = gz_buffer.getvalue()

    # Encrypt with age
    encrypted_data = pyrage.encrypt(compressed_data, [recipient])

    # Write to file
    with open(INPUTS_ARCHIVE, "wb") as f:
        f.write(encrypted_data)

    click.echo(f"Encrypted inputs to {INPUTS_ARCHIVE}")
