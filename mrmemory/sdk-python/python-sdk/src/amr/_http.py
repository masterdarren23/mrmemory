"""HTTP transport with retry logic for AMR."""

from __future__ import annotations

import random
import time
from typing import Any

import httpx

from amr._config import Config
from amr.errors import (
    AMRError,
    AuthenticationError,
    NotFoundError,
    RateLimitError,
    ServerError,
    ValidationError,
)

_RETRYABLE_STATUS = {429, 500, 502, 503, 504}


def _raise_for_status(response: httpx.Response) -> None:
    """Raise an appropriate AMRError for non-2xx responses."""
    code = response.status_code
    if 200 <= code < 300:
        return

    try:
        body = response.json()
        message = body.get("error", body.get("message", response.text))
    except Exception:
        message = response.text or f"HTTP {code}"

    if code == 401:
        raise AuthenticationError(message, status_code=401)
    if code == 404:
        raise NotFoundError(message)
    if code == 422:
        raise ValidationError(message)
    if code == 429:
        retry_after = float(response.headers.get("retry-after", "1"))
        raise RateLimitError(message, retry_after=retry_after)
    if code >= 500:
        raise ServerError(message, status_code=code)
    raise AMRError(message, status_code=code)


def _backoff(attempt: int) -> float:
    """Exponential backoff with jitter."""
    base = min(2**attempt, 30)
    return base * (0.5 + random.random() * 0.5)


class SyncTransport:
    """Synchronous HTTP transport."""

    def __init__(self, config: Config) -> None:
        self._config = config
        self._client = httpx.Client(
            base_url=config.base_url,
            headers={
                "Authorization": f"Bearer {config.api_key}",
                "Content-Type": "application/json",
                "User-Agent": "amr-python/0.1.0",
            },
            timeout=config.timeout,
        )

    def request(self, method: str, path: str, **kwargs: Any) -> Any:
        """Make an HTTP request with automatic retries."""
        last_exc: Exception | None = None
        for attempt in range(self._config.max_retries + 1):
            try:
                response = self._client.request(method, path, **kwargs)
                if response.status_code in _RETRYABLE_STATUS and attempt < self._config.max_retries:
                    wait = float(response.headers.get("retry-after", "0")) or _backoff(attempt)
                    time.sleep(wait)
                    continue
                _raise_for_status(response)
                if response.status_code == 204:
                    return None
                return response.json()
            except (httpx.ConnectError, httpx.ReadTimeout) as exc:
                last_exc = exc
                if attempt < self._config.max_retries:
                    time.sleep(_backoff(attempt))
                    continue
                raise AMRError(f"Connection failed: {exc}") from exc
        raise last_exc or AMRError("Request failed after retries")  # type: ignore[misc]

    def close(self) -> None:
        self._client.close()


class AsyncTransport:
    """Asynchronous HTTP transport."""

    def __init__(self, config: Config) -> None:
        self._config = config
        self._client = httpx.AsyncClient(
            base_url=config.base_url,
            headers={
                "Authorization": f"Bearer {config.api_key}",
                "Content-Type": "application/json",
                "User-Agent": "amr-python/0.1.0",
            },
            timeout=config.timeout,
        )

    async def request(self, method: str, path: str, **kwargs: Any) -> Any:
        """Make an async HTTP request with automatic retries."""
        import asyncio

        last_exc: Exception | None = None
        for attempt in range(self._config.max_retries + 1):
            try:
                response = await self._client.request(method, path, **kwargs)
                if response.status_code in _RETRYABLE_STATUS and attempt < self._config.max_retries:
                    wait = float(response.headers.get("retry-after", "0")) or _backoff(attempt)
                    await asyncio.sleep(wait)
                    continue
                _raise_for_status(response)
                if response.status_code == 204:
                    return None
                return response.json()
            except (httpx.ConnectError, httpx.ReadTimeout) as exc:
                last_exc = exc
                if attempt < self._config.max_retries:
                    await asyncio.sleep(_backoff(attempt))
                    continue
                raise AMRError(f"Connection failed: {exc}") from exc
        raise last_exc or AMRError("Request failed after retries")  # type: ignore[misc]

    async def close(self) -> None:
        await self._client.aclose()
