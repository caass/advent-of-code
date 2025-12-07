"""Decrypt AoC puzzle inputs from the encrypted archive."""

import gzip
import io
import os
import tarfile

import click
import pyrage

from ..paths import INPUTS_ARCHIVE, INPUTS_DIR, ROOT_DIR


def decrypt_inputs() -> None:
    """
    Decrypt puzzle inputs from inputs.gz.age to target/inputs/.

    Requires the AOC_INPUTS_SECRET environment variable to be set to the
    age identity (private key).
    """
    secret = os.environ.get("AOC_INPUTS_SECRET")
    if not secret:
        raise click.ClickException(
            "Need AOC_INPUTS_SECRET to be set to decrypt puzzle inputs."
        )

    # Parse the identity from the secret
    identity = pyrage.x25519.Identity.from_str(secret.strip())

    # Read and decrypt the archive
    with open(INPUTS_ARCHIVE, "rb") as f:
        encrypted_data = f.read()

    decrypted_data = pyrage.decrypt(encrypted_data, [identity])

    # Decompress and extract
    with gzip.GzipFile(fileobj=io.BytesIO(decrypted_data)) as gz:
        with tarfile.open(fileobj=gz, mode="r:") as tar:
            # Extract to ROOT_DIR since the archive contains ./target/inputs
            tar.extractall(ROOT_DIR, filter="data")

    click.echo(f"Decrypted inputs to {INPUTS_DIR}")
