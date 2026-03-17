"""AMR — Agent Memory Relay. Persistent long-term memory for AI agents."""

from amr.client import AMR
from amr.async_client import AsyncAMR
from amr.types import Memory, MemoryEvent
from amr.errors import (
    AMRError,
    AuthenticationError,
    RateLimitError,
    NotFoundError,
    ValidationError,
    ServerError,
)

__all__ = [
    "AMR",
    "AsyncAMR",
    "Memory",
    "MemoryEvent",
    "AMRError",
    "AuthenticationError",
    "RateLimitError",
    "NotFoundError",
    "ValidationError",
    "ServerError",
]

__version__ = "0.1.0"
