"""Configuration resolution for AMR clients."""

from __future__ import annotations

import os
from dataclasses import dataclass

DEFAULT_BASE_URL = "https://api.amr.dev/v1"
DEFAULT_TIMEOUT = 10.0
DEFAULT_MAX_RETRIES = 3


@dataclass(slots=True)
class Config:
    """Resolved client configuration."""

    api_key: str
    base_url: str
    agent_id: str | None
    namespace: str | None
    timeout: float
    max_retries: int

    @classmethod
    def resolve(
        cls,
        api_key: str | None = None,
        base_url: str | None = None,
        agent_id: str | None = None,
        namespace: str | None = None,
        timeout: float | None = None,
        max_retries: int | None = None,
    ) -> Config:
        """Build config from explicit args → env vars → defaults."""
        resolved_key = api_key or os.environ.get("AMR_API_KEY", "")
        if not resolved_key:
            raise ValueError(
                "No API key provided. Pass api_key= or set AMR_API_KEY environment variable."
            )
        return cls(
            api_key=resolved_key,
            base_url=(base_url or os.environ.get("AMR_BASE_URL") or DEFAULT_BASE_URL).rstrip("/"),
            agent_id=agent_id or os.environ.get("AMR_AGENT_ID"),
            namespace=namespace or os.environ.get("AMR_NAMESPACE"),
            timeout=timeout if timeout is not None else DEFAULT_TIMEOUT,
            max_retries=max_retries if max_retries is not None else DEFAULT_MAX_RETRIES,
        )
