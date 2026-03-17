"""AMR data types."""

from __future__ import annotations

from dataclasses import dataclass, field
from datetime import datetime, timedelta
from typing import Literal


@dataclass(frozen=True, slots=True)
class Memory:
    """A stored memory."""

    id: str
    content: str
    tags: list[str] = field(default_factory=list)
    namespace: str = "default"
    agent_id: str = ""
    score: float | None = None
    ttl: timedelta | None = None
    expires_at: datetime | None = None
    created_at: datetime = field(default_factory=datetime.now)
    updated_at: datetime = field(default_factory=datetime.now)

    @classmethod
    def from_dict(cls, data: dict) -> Memory:  # type: ignore[type-arg]
        """Parse a Memory from an API response dict."""
        return cls(
            id=data["id"],
            content=data["content"],
            tags=data.get("tags", []),
            namespace=data.get("namespace", "default"),
            agent_id=data.get("agent_id", ""),
            score=data.get("score"),
            ttl=timedelta(seconds=data["ttl"]) if data.get("ttl") else None,
            expires_at=_parse_dt(data.get("expires_at")),
            created_at=_parse_dt(data.get("created_at")) or datetime.now(),
            updated_at=_parse_dt(data.get("updated_at")) or datetime.now(),
        )


@dataclass(frozen=True, slots=True)
class MemoryEvent:
    """A real-time memory event from the WebSocket stream."""

    type: Literal["memory.created", "memory.shared", "memory.expired"]
    memory_id: str
    memory: Memory | None = None
    timestamp: datetime = field(default_factory=datetime.now)

    @classmethod
    def from_dict(cls, data: dict) -> MemoryEvent:  # type: ignore[type-arg]
        """Parse a MemoryEvent from a WebSocket message."""
        mem = Memory.from_dict(data["memory"]) if data.get("memory") else None
        return cls(
            type=data["type"],
            memory_id=data.get("memory_id", mem.id if mem else ""),
            memory=mem,
            timestamp=_parse_dt(data.get("timestamp")) or datetime.now(),
        )


def _parse_dt(value: str | None) -> datetime | None:
    """Parse an ISO 8601 datetime string."""
    if value is None:
        return None
    # Handle trailing Z
    s = value.replace("Z", "+00:00")
    return datetime.fromisoformat(s)
