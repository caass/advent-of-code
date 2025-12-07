"""Browser detection and session cookie retrieval for AoC."""

import platform
import subprocess
from enum import StrEnum
from typing import Callable

import browser_cookie3
import click
from http.cookiejar import CookieJar


class Browser(StrEnum):
    """
    Supported browsers for cookie extraction.

    Each variant knows how to retrieve cookies from its corresponding browser
    and the class can detect the system's default browser.
    """

    FIREFOX = "firefox"
    CHROME = "chrome"
    CHROMIUM = "chromium"
    EDGE = "edge"
    SAFARI = "safari"

    def _cookie_function(self) -> Callable[..., CookieJar]:
        """Get the browser_cookie3 function for this browser."""
        return {
            Browser.FIREFOX: browser_cookie3.firefox,
            Browser.CHROME: browser_cookie3.chrome,
            Browser.CHROMIUM: browser_cookie3.chromium,
            Browser.EDGE: browser_cookie3.edge,
            Browser.SAFARI: browser_cookie3.safari,
        }[self]

    def session_cookie(self, domain: str = ".adventofcode.com") -> str | None:
        """
        Attempt to retrieve a session cookie from this browser.

        Args:
            domain: The domain to get cookies for.

        Returns:
            The session cookie value, or None if not found.
        """
        try:
            cookies = self._cookie_function()(domain_name=domain)
            for cookie in cookies:
                if cookie.name == "session":
                    return cookie.value
        except Exception:
            pass
        return None

    @staticmethod
    def default() -> "Browser | None":
        """
        Detect the system's default browser.

        On macOS, uses LaunchServices to query the default HTTPS handler.
        On Linux, uses xdg-settings to query the default web browser.

        Returns:
            The Browser enum value if detected, None otherwise.
        """
        system = platform.system()

        if system == "Darwin":
            return Browser._default_macos()
        elif system == "Linux":
            return Browser._default_linux()

        return None

    @staticmethod
    def _default_macos() -> "Browser | None":
        """Detect default browser on macOS using LaunchServices."""
        try:
            from LaunchServices import LSCopyDefaultHandlerForURLScheme  # type: ignore[import-not-found]

            bundle_id = LSCopyDefaultHandlerForURLScheme("https")
            if bundle_id:
                bundle_id = bundle_id.lower()
                if "firefox" in bundle_id:
                    return Browser.FIREFOX
                elif "chrome" in bundle_id and "chromium" not in bundle_id:
                    return Browser.CHROME
                elif "chromium" in bundle_id:
                    return Browser.CHROMIUM
                elif "safari" in bundle_id:
                    return Browser.SAFARI
                elif "edge" in bundle_id:
                    return Browser.EDGE
        except ImportError:
            pass
        return None

    @staticmethod
    def _default_linux() -> "Browser | None":
        """Detect default browser on Linux using xdg-settings."""
        try:
            result = subprocess.run(
                ["xdg-settings", "get", "default-web-browser"],
                capture_output=True,
                text=True,
            )
            if result.returncode == 0:
                browser = result.stdout.strip().lower()
                if "firefox" in browser:
                    return Browser.FIREFOX
                elif "chrome" in browser and "chromium" not in browser:
                    return Browser.CHROME
                elif "chromium" in browser:
                    return Browser.CHROMIUM
                elif "edge" in browser:
                    return Browser.EDGE
        except FileNotFoundError:
            pass
        return None


def validate_browser(
    ctx: click.Context, param: click.Parameter, value: str | None
) -> Browser | None:
    """
    Click callback to validate and convert a browser string to Browser enum.

    Used with @click.option to convert the string value from the CLI
    to the Browser enum while providing helpful error messages for invalid values.
    """
    if value is None:
        return None

    try:
        return Browser(value.lower())
    except ValueError:
        raise click.BadParameter(
            f"Invalid browser: {value}. Must be one of: {', '.join(b.value for b in Browser)}",
            ctx=ctx,
            param=param,
        )


def get_session_cookie(browser: Browser | None = None) -> str:
    """
    Get the AoC session cookie from a browser.

    Args:
        browser: Specific browser to use. If None, tries the default browser
                 first, then falls back to trying all browsers.

    Returns:
        The session cookie value.

    Raises:
        click.ClickException: If the cookie cannot be found in any browser.
    """
    browsers_to_try: list[Browser] = []

    if browser is not None:
        browsers_to_try = [browser]
    else:
        # Try default browser first
        default = Browser.default()
        if default:
            browsers_to_try.append(default)

        # Then try all others
        for b in Browser:
            if b not in browsers_to_try:
                browsers_to_try.append(b)

    for b in browsers_to_try:
        cookie = b.session_cookie()
        if cookie:
            click.echo(f"Found session cookie in {b.value}")
            return cookie

    raise click.ClickException(
        "Could not find AoC session cookie in any browser. "
        "Please log in to adventofcode.com in your browser first."
    )
