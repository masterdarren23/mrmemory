"""Synchronous AMR client."""

from __future__ import annotations

from datetime import timedelta
from typing import Any

from amr._config import Config
from amr._http import SyncTransport
from amr.types import Memory


class AMR:
    """Synchronous client for Agent Memory Relay.

    Usage::

        from amr import AMR

        amr = AMR("amr_sk_...")
        amr.remember("User prefers dark mode", tags=["preferences"])
        memories = amr.recall("What does the user prefer?")
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
        self._transport = SyncTransport(self._config)

    # -- Context manager --

    def __enter__(self) -> AMR:
        return self

    def __exit__(self, *_: Any) -> None:
        self.close()

    def close(self) -> None:
        """Close the underlying HTTP connection pool."""
        self._transport.close()

    # -- Core API --

    def remember(
        self,
        content: str,
        *,
        tags: list[str] | None = None,
        namespace: str | None = None,
        agent_id: str | None = None,
        ttl: timedelta | None = None,
    ) -> Memory:
        """Store a memory.

        Args:
            content: The memory text to store.
            tags: Optional tags for filtering.
            namespace: Override the client default namespace.
            agent_id: Override the client default agent ID.
            ttl: Auto-expire after this duration.

        Returns:
            The created Memory object.
        """
        body: dict[str, Any] = {"content": content}
        if tags:
            body["tags"] = tags
        if namespace or self._config.namespace:
            body["namespace"] = namespace or self._config.namespace
        if agent_id or self._config.agent_id:
            body["agent_id"] = agent_id or self._config.agent_id
        if ttl is not None:
            body["ttl"] = int(ttl.total_seconds())

        data = self._transport.request("POST", "/remember", json=body)
        return Memory.from_dict(data)

    def recall(
        self,
        query: str,
        *,
        namespace: str | None = None,
        agent_id: str | None = None,
        tags: list[str] | None = None,
        limit: int = 5,
        threshold: float = 0.7,
    ) -> list[Memory]:
        """Retrieve relevant memories via semantic search.

        Args:
            query: Natural language query.
            namespace: Filter by namespace.
            agent_id: Filter by agent ID.
            tags: Filter by tags (AND).
            limit: Maximum results (default 5).
            threshold: Minimum similarity score (default 0.7).

        Returns:
            List of Memory objects ranked by relevance.
        """
        body: dict[str, Any] = {"query": query, "limit": limit, "threshold": threshold}
        if tags:
            body["tags"] = tags
        if namespace or self._config.namespace:
            body["namespace"] = namespace or self._config.namespace
        if agent_id or self._config.agent_id:
            body["agent_id"] = agent_id or self._config.agent_id

        data = self._transport.request("POST", "/recall", json=body)
        return [Memory.from_dict(m) for m in data.get("memories", data if isinstance(data, list) else [])]

    def share(
        self,
        memory_ids: str | list[str],
        *,
        target_agent: str,
        permissions: str = "read",
    ) -> None:
        """Share memories with another agent.

        Args:
            memory_ids: Single ID or list of memory IDs to share.
            target_agent: The target agent ID.
            permissions: "read" (default) or "readwrite".
        """
        ids = [memory_ids] if isinstance(memory_ids, str) else memory_ids
        self._transport.request(
            "POST",
            "/share",
            json={
                "memory_ids": ids,
                "target_agent_id": target_agent,
                "permissions": permissions,
            },
        )

    def forget(self, memory_ids: str | list[str]) -> None:
        """Delete one or more memories.

        Args:
            memory_ids: Single ID or list of memory IDs to delete.
        """
        ids = [memory_ids] if isinstance(memory_ids, str) else memory_ids
        self._transport.request("DELETE", "/forget", json={"memory_ids": ids})

    def forget_all(self, *, namespace: str | None = None) -> None:
        """Delete all memories, optionally scoped to a namespace.

        Args:
            namespace: If provided, only delete memories in this namespace.
        """
        body: dict[str, Any] = {"forget_all": True}
        if namespace:
            body["namespace"] = namespace
        self._transport.request("DELETE", "/forget", json=body)

    def memories(
        self,
        *,
        namespace: str | None = None,
        agent_id: str | None = None,
        tags: list[str] | None = None,
        limit: int = 20,
        offset: int = 0,
    ) -> list[Memory]:
        """List memories with optional filters.

        Args:
            namespace: Filter by namespace.
            agent_id: Filter by agent ID.
            tags: Filter by tags.
            limit: Page size (default 20).
            offset: Pagination offset.

        Returns:
            List of Memory objects.
        """
        params: dict[str, Any] = {"limit": limit, "offset": offset}
        if namespace or self._config.namespace:
            params["namespace"] = namespace or self._config.namespace
        if agent_id or self._config.agent_id:
            params["agent_id"] = agent_id or self._config.agent_id
        if tags:
            params["tags"] = ",".join(tags)

        data = self._transport.request("GET", "/memories", params=params)
        return [Memory.from_dict(m) for m in data.get("memories", data if isinstance(data, list) else [])]

    def auto_remember(
        self,
        messages: list[dict[str, str]],
        *,
        namespace: str | None = None,
        agent_id: str | None = None,
        llm_api_key: str | None = None,
        sync: bool = False,
    ) -> dict[str, Any]:
        """Extract and store memories from conversation messages using LLM.

        The server-side LLM extracts facts, preferences, and entities from the
        conversation, deduplicates against existing memories, and stores new ones.

        Args:
            messages: List of conversation messages, each with ``role`` and ``content``.
            namespace: Override the client default namespace.
            agent_id: Override the client default agent ID.
            llm_api_key: BYOK — provide your own OpenAI API key for extraction.
            sync: If True, block until extraction completes. Default is async (fire-and-forget).

        Returns:
            Dict with ``job_id`` and ``status`` (async) or extracted ``memories`` (sync).
        """
        body: dict[str, Any] = {"messages": messages}
        if namespace or self._config.namespace:
            body["namespace"] = namespace or self._config.namespace
        if agent_id or self._config.agent_id:
            body["agent_id"] = agent_id or self._config.agent_id
        if llm_api_key:
            body["llm_api_key"] = llm_api_key
        if sync:
            body["sync"] = True

        return self._transport.request("POST", "/memories/auto", json=body)

    def compress(
        self,
        *,
        namespace: str | None = None,
        agent_id: str | None = None,
        llm_api_key: str | None = None,
        threshold: int = 10,
        similarity_threshold: float = 0.75,
        sync: bool = False,
        dry_run: bool = False,
    ) -> dict[str, Any]:
        """Compress related memories in a namespace using LLM summarization.

        Groups semantically similar memories and merges each group into a single,
        denser memory. Originals are replaced with the compressed version.

        Args:
            namespace: Namespace to compress (default: client default).
            agent_id: Override the client default agent ID.
            llm_api_key: BYOK — provide your own OpenAI API key.
            threshold: Minimum memory count before compression triggers (default 10).
            similarity_threshold: Cosine similarity threshold for grouping (default 0.75).
            sync: If True, block until compression completes.
            dry_run: If True, return what would be compressed without doing it.

        Returns:
            Dict with compression results (groups, counts, etc.).
        """
        body: dict[str, Any] = {}
        if namespace or self._config.namespace:
            body["namespace"] = namespace or self._config.namespace
        if agent_id or self._config.agent_id:
            body["agent_id"] = agent_id or self._config.agent_id
        if llm_api_key:
            body["llm_api_key"] = llm_api_key
        body["threshold"] = threshold
        body["similarity_threshold"] = similarity_threshold
        if sync:
            body["sync"] = True
        if dry_run:
            body["dry_run"] = True

        return self._transport.request("POST", "/memories/compress", json=body)
