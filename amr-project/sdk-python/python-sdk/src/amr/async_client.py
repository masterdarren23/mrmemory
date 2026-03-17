"""Asynchronous AMR client."""

from __future__ import annotations

from datetime import timedelta
from typing import Any

from amr._config import Config
from amr._http import AsyncTransport
from amr.types import Memory


class AsyncAMR:
    """Asynchronous client for Agent Memory Relay.

    Usage::

        from amr import AsyncAMR

        async with AsyncAMR("amr_sk_...") as amr:
            await amr.remember("User prefers dark mode", tags=["preferences"])
            memories = await amr.recall("What does the user prefer?")
    """

    def __init__(
        self,
        api_key: str | None = None,
        *,
        base_url: str | None = None,
        agent_id: str | None = None,
        namespace: str | None = None,
        timeout: float | None = None,
        max_retries: int | None = None,
    ) -> None:
        self._config = Config.resolve(
            api_key=api_key,
            base_url=base_url,
            agent_id=agent_id,
            namespace=namespace,
            timeout=timeout,
            max_retries=max_retries,
        )
        self._transport = AsyncTransport(self._config)

    # -- Context manager --

    async def __aenter__(self) -> AsyncAMR:
        return self

    async def __aexit__(self, *_: Any) -> None:
        await self.close()

    async def close(self) -> None:
        """Close the underlying HTTP connection pool."""
        await self._transport.close()

    # -- Core API --

    async def remember(
        self,
        content: str,
        *,
        tags: list[str] | None = None,
        namespace: str | None = None,
        agent_id: str | None = None,
        ttl: timedelta | None = None,
    ) -> Memory:
        """Store a memory."""
        body: dict[str, Any] = {"content": content}
        if tags:
            body["tags"] = tags
        if namespace or self._config.namespace:
            body["namespace"] = namespace or self._config.namespace
        if agent_id or self._config.agent_id:
            body["agent_id"] = agent_id or self._config.agent_id
        if ttl is not None:
            body["ttl"] = int(ttl.total_seconds())

        data = await self._transport.request("POST", "/remember", json=body)
        return Memory.from_dict(data)

    async def recall(
        self,
        query: str,
        *,
        namespace: str | None = None,
        agent_id: str | None = None,
        tags: list[str] | None = None,
        limit: int = 5,
        threshold: float = 0.7,
    ) -> list[Memory]:
        """Retrieve relevant memories via semantic search."""
        body: dict[str, Any] = {"query": query, "limit": limit, "threshold": threshold}
        if tags:
            body["tags"] = tags
        if namespace or self._config.namespace:
            body["namespace"] = namespace or self._config.namespace
        if agent_id or self._config.agent_id:
            body["agent_id"] = agent_id or self._config.agent_id

        data = await self._transport.request("POST", "/recall", json=body)
        return [Memory.from_dict(m) for m in data.get("memories", data if isinstance(data, list) else [])]

    async def share(
        self,
        memory_ids: str | list[str],
        *,
        target_agent: str,
        permissions: str = "read",
    ) -> None:
        """Share memories with another agent."""
        ids = [memory_ids] if isinstance(memory_ids, str) else memory_ids
        await self._transport.request(
            "POST",
            "/share",
            json={
                "memory_ids": ids,
                "target_agent_id": target_agent,
                "permissions": permissions,
            },
        )

    async def forget(self, memory_ids: str | list[str]) -> None:
        """Delete one or more memories."""
        ids = [memory_ids] if isinstance(memory_ids, str) else memory_ids
        await self._transport.request("DELETE", "/forget", json={"memory_ids": ids})

    async def forget_all(self, *, namespace: str | None = None) -> None:
        """Delete all memories, optionally scoped to a namespace."""
        body: dict[str, Any] = {"forget_all": True}
        if namespace:
            body["namespace"] = namespace
        await self._transport.request("DELETE", "/forget", json=body)

    async def memories(
        self,
        *,
        namespace: str | None = None,
        agent_id: str | None = None,
        tags: list[str] | None = None,
        limit: int = 20,
        offset: int = 0,
    ) -> list[Memory]:
        """List memories with optional filters."""
        params: dict[str, Any] = {"limit": limit, "offset": offset}
        if namespace or self._config.namespace:
            params["namespace"] = namespace or self._config.namespace
        if agent_id or self._config.agent_id:
            params["agent_id"] = agent_id or self._config.agent_id
        if tags:
            params["tags"] = ",".join(tags)

        data = await self._transport.request("GET", "/memories", params=params)
        return [Memory.from_dict(m) for m in data.get("memories", data if isinstance(data, list) else [])]
