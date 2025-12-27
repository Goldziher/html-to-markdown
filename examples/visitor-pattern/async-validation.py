#!/usr/bin/env python3
"""
Asynchronous URL Validation Example.

Demonstrates how to validate URLs asynchronously during conversion.
Uses aiohttp to check HTTP status codes of all links and images.
Fails conversion if any URLs are broken (404, 500, etc.).
"""

import asyncio

import aiohttp

from html_to_markdown import convert_with_async_visitor


class AsyncUrlValidator:
    """Validates all URLs asynchronously using HTTP HEAD requests."""

    def __init__(self, timeout: int = 5) -> None:
        self.timeout = timeout
        self.session: aiohttp.ClientSession | None = None
        self.validated_urls = []

    async def visit_link(self, ctx, href: str, text: str, title: str | None):
        """Validate link URLs."""
        # Skip anchor links
        if href.startswith("#"):
            return {"type": "continue"}

        # Skip mailto and tel links
        if href.startswith(("mailto:", "tel:")):
            return {"type": "continue"}

        # Validate HTTP(S) URLs
        if href.startswith(("http://", "https://")):
            is_valid, status = await self._validate_url(href)
            self.validated_urls.append((href, status))

            if not is_valid:
                return {"type": "error", "message": f"Broken link ({status}): {href}"}

        return {"type": "continue"}

    async def visit_image(self, ctx, src: str, alt: str | None, title: str | None):
        """Validate image URLs."""
        # Skip data URIs and relative paths
        if not src.startswith(("http://", "https://")):
            return {"type": "continue"}

        is_valid, status = await self._validate_url(src)
        self.validated_urls.append((src, status))

        if not is_valid:
            return {"type": "error", "message": f"Broken image ({status}): {src}"}

        return {"type": "continue"}

    async def _validate_url(self, url: str) -> tuple[bool, int | str]:
        """Validate a URL using HTTP HEAD request."""
        if self.session is None:
            self.session = aiohttp.ClientSession()

        try:
            async with self.session.head(url, timeout=self.timeout, allow_redirects=True) as response:
                # Consider 2xx and 3xx as valid
                is_valid = response.status < 400
                return (is_valid, response.status)
        except asyncio.TimeoutError:
            return (False, "timeout")
        except aiohttp.ClientError as e:
            return (False, f"error: {type(e).__name__}")
        except Exception as e:
            return (False, f"error: {e!s}")

    async def close(self) -> None:
        """Close the HTTP session."""
        if self.session:
            await self.session.close()


async def test_valid_urls() -> bool | None:
    """Test with valid URLs (should pass)."""
    html = """
    <h1>Article with Valid Links</h1>
    <p>Check out <a href="https://www.example.com">Example</a>.</p>
    <img src="https://via.placeholder.com/150" alt="Placeholder image">
    <p>Internal anchor: <a href="#section1">Section 1</a></p>
    """

    visitor = AsyncUrlValidator(timeout=10)
    try:
        await convert_with_async_visitor(html, visitor=visitor)
        for _url, _status in visitor.validated_urls:
            pass
        return True
    except Exception:
        return False
    finally:
        await visitor.close()


async def test_broken_link() -> bool | None:
    """Test with broken URL (should fail)."""
    html = """
    <h1>Article with Broken Link</h1>
    <p>This link is broken: <a href="https://httpstat.us/404">404 page</a>.</p>
    """

    visitor = AsyncUrlValidator(timeout=10)
    try:
        await convert_with_async_visitor(html, visitor=visitor)
        return False
    except Exception:
        for _url, _status in visitor.validated_urls:
            pass
        return True
    finally:
        await visitor.close()


async def main() -> None:
    tests = [
        ("Valid URLs", test_valid_urls),
        ("Broken Link (404)", test_broken_link),
    ]

    results = []
    for name, test_func in tests:
        passed = await test_func()
        results.append((name, passed))

    for name, passed in results:
        pass


if __name__ == "__main__":
    asyncio.run(main())
