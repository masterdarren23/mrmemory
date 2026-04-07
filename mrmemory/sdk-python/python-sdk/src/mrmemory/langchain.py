"""LangGraph checkpoint and store integrations backed by MrMemory (AMR).

Requires optional dependencies::

    pip install mrmemory[langchain]

Usage::

    from mrmemory.langchain import MrMemoryCheckpointer, MrMemoryStore
    from langgraph.graph import StateGraph

    checkpointer = MrMemoryCheckpointer(api_key="amr_sk_...")
    store = MrMemoryStore(api_key="amr_sk_...")

    graph = StateGraph(...).compile(checkpointer=checkpointer, store=store)
"""

from __future__ import annotations

import json
from typing import Any, Iterator, Optional, Sequence, Tuple

try:
    from langgraph.checkpoint.base import (
        BaseCheckpointSaver,
        Checkpoint,
        CheckpointMetadata,
        CheckpointTuple,
    )
    from langgraph.store.base import BaseStore, Item, GetOp, PutOp, SearchOp, ListNamespacesOp
except ImportError as _exc:
    raise ImportError(
        "LangGraph integration requires langgraph. Install with: pip install mrmemory[langchain]"
    ) from _exc

from amr.client import AMR


# ---------------------------------------------------------------------------
# Checkpointer
# ---------------------------------------------------------------------------


class MrMemoryCheckpointer(BaseCheckpointSaver):
    """LangGraph checkpointer that persists state to MrMemory.

    Each LangGraph *thread* maps to an AMR namespace prefixed with
    ``langgraph:cp:<thread_id>``.  Checkpoints are stored as JSON memories
    with metadata containing ``checkpoint_id`` and ``parent_checkpoint_id``.

    Args:
        api_key: AMR API key (``amr_sk_...``).
        base_url: Override the default API URL.
        agent_id: Optional agent ID scope.
        **kwargs: Passed to ``BaseCheckpointSaver``.

    Example::

        from mrmemory.langchain import MrMemoryCheckpointer

        cp = MrMemoryCheckpointer(api_key="amr_sk_...")
        graph = builder.compile(checkpointer=cp)
        graph.invoke({"messages": [...]}, config={"configurable": {"thread_id": "t1"}})
    """

    def __init__(
        self,
        api_key: str | None = None,
        *,
        base_url: str | None = None,
        agent_id: str | None = None,
        **kwargs: Any,
    ) -> None:
        super().__init__(**kwargs)
        self._client = AMR(api_key=api_key, base_url=base_url, agent_id=agent_id)

    # -- helpers --

    @staticmethod
    def _ns(thread_id: str) -> str:
        return f"langgraph:cp:{thread_id}"

    # -- interface --

    def put(
        self,
        config: dict[str, Any],
        checkpoint: Checkpoint,
        metadata: CheckpointMetadata,
        new_versions: Any = None,
    ) -> dict[str, Any]:
        thread_id = config["configurable"]["thread_id"]
        checkpoint_ns = config["configurable"].get("checkpoint_ns", "")
        checkpoint_id = checkpoint["id"]

        parent_id = config["configurable"].get("checkpoint_id")

        payload = {
            "checkpoint": checkpoint,
            "metadata": metadata,
            "parent_checkpoint_id": parent_id,
            "checkpoint_ns": checkpoint_ns,
        }

        self._client.remember(
            json.dumps(payload, default=str),
            namespace=self._ns(thread_id),
            tags=["langgraph_checkpoint", f"cpid:{checkpoint_id}"],
        )

        return {
            "configurable": {
                "thread_id": thread_id,
                "checkpoint_ns": checkpoint_ns,
                "checkpoint_id": checkpoint_id,
            }
        }

    def get_tuple(self, config: dict[str, Any]) -> Optional[CheckpointTuple]:
        thread_id = config["configurable"]["thread_id"]
        checkpoint_id = config["configurable"].get("checkpoint_id")

        memories = self._client.memories(namespace=self._ns(thread_id), limit=50)
        if not memories:
            return None

        # Filter to checkpoint memories and find the right one
        candidates = []
        for m in memories:
            try:
                data = json.loads(m.content)
            except (json.JSONDecodeError, TypeError):
                continue
            if "checkpoint" not in data:
                continue
            candidates.append((m, data))

        if not candidates:
            return None

        if checkpoint_id:
            for m, data in candidates:
                if data["checkpoint"].get("id") == checkpoint_id:
                    break
            else:
                return None
        else:
            # Latest by created_at
            m, data = max(candidates, key=lambda x: x[0].created_at)

        cp = data["checkpoint"]
        md = data.get("metadata", {})
        parent_id = data.get("parent_checkpoint_id")
        checkpoint_ns = data.get("checkpoint_ns", "")

        parent_config = None
        if parent_id:
            parent_config = {
                "configurable": {
                    "thread_id": thread_id,
                    "checkpoint_ns": checkpoint_ns,
                    "checkpoint_id": parent_id,
                }
            }

        return CheckpointTuple(
            config={
                "configurable": {
                    "thread_id": thread_id,
                    "checkpoint_ns": checkpoint_ns,
                    "checkpoint_id": cp["id"],
                }
            },
            checkpoint=cp,
            metadata=md,
            parent_config=parent_config,
        )

    def list(
        self,
        config: Optional[dict[str, Any]],
        *,
        filter: Optional[dict[str, Any]] = None,
        before: Optional[dict[str, Any]] = None,
        limit: int = 10,
    ) -> Iterator[CheckpointTuple]:
        if config is None:
            return

        thread_id = config["configurable"]["thread_id"]
        memories = self._client.memories(namespace=self._ns(thread_id), limit=limit)

        for m in memories:
            try:
                data = json.loads(m.content)
            except (json.JSONDecodeError, TypeError):
                continue
            if "checkpoint" not in data:
                continue

            cp = data["checkpoint"]
            md = data.get("metadata", {})
            parent_id = data.get("parent_checkpoint_id")
            checkpoint_ns = data.get("checkpoint_ns", "")

            parent_config = None
            if parent_id:
                parent_config = {
                    "configurable": {
                        "thread_id": thread_id,
                        "checkpoint_ns": checkpoint_ns,
                        "checkpoint_id": parent_id,
                    }
                }

            yield CheckpointTuple(
                config={
                    "configurable": {
                        "thread_id": thread_id,
                        "checkpoint_ns": checkpoint_ns,
                        "checkpoint_id": cp["id"],
                    }
                },
                checkpoint=cp,
                metadata=md,
                parent_config=parent_config,
            )


# ---------------------------------------------------------------------------
# Store
# ---------------------------------------------------------------------------


class MrMemoryStore(BaseStore):
    """LangGraph Store backed by MrMemory for cross-thread shared memory.

    Namespace tuples are joined with ``:`` to form AMR namespace strings.
    Search uses AMR's semantic recall endpoint.

    Args:
        api_key: AMR API key.
        base_url: Override the default API URL.
        agent_id: Optional agent ID scope.

    Example::

        from mrmemory.langchain import MrMemoryStore

        store = MrMemoryStore(api_key="amr_sk_...")
        store.put(("user", "prefs"), "theme", {"value": "dark"})
        results = store.search(("user", "prefs"), query="theme preference")
    """

    def __init__(
        self,
        api_key: str | None = None,
        *,
        base_url: str | None = None,
        agent_id: str | None = None,
    ) -> None:
        self._client = AMR(api_key=api_key, base_url=base_url, agent_id=agent_id)

    @staticmethod
    def _ns(namespace: Tuple[str, ...]) -> str:
        return ":".join(namespace) if namespace else "default"

    def batch(self, ops: Sequence[GetOp | PutOp | SearchOp | ListNamespacesOp]) -> list[Any]:
        results: list[Any] = []
        for op in ops:
            if isinstance(op, GetOp):
                results.append(self._get(op.namespace, op.key))
            elif isinstance(op, PutOp):
                self._put(op.namespace, op.key, op.value)
                results.append(None)
            elif isinstance(op, SearchOp):
                results.append(self._search(op.namespace_prefix, op.query, op.limit))
            elif isinstance(op, ListNamespacesOp):
                # Not efficiently supported — return empty
                results.append([])
            else:
                results.append(None)
        return results

    async def abatch(self, ops: Sequence[GetOp | PutOp | SearchOp | ListNamespacesOp]) -> list[Any]:
        # For now, delegate to sync
        return self.batch(ops)

    def _get(self, namespace: Tuple[str, ...], key: str) -> Optional[Item]:
        ns = self._ns(namespace)
        memories = self._client.memories(namespace=ns, tags=[f"key:{key}"], limit=1)
        if not memories:
            return None
        m = memories[0]
        try:
            value = json.loads(m.content)
        except (json.JSONDecodeError, TypeError):
            value = {"content": m.content}
        return Item(
            value=value,
            key=key,
            namespace=namespace,
            created_at=m.created_at,
            updated_at=m.updated_at,
        )

    def _put(self, namespace: Tuple[str, ...], key: str, value: Optional[dict[str, Any]]) -> None:
        ns = self._ns(namespace)
        if value is None:
            # Delete: find and forget
            memories = self._client.memories(namespace=ns, tags=[f"key:{key}"], limit=1)
            if memories:
                self._client.forget(memories[0].id)
            return

        # Upsert: delete old then create new
        memories = self._client.memories(namespace=ns, tags=[f"key:{key}"], limit=1)
        if memories:
            self._client.forget(memories[0].id)

        self._client.remember(
            json.dumps(value, default=str),
            namespace=ns,
            tags=[f"key:{key}", "langgraph_store"],
        )

    def _search(
        self,
        namespace_prefix: Tuple[str, ...],
        query: Optional[str],
        limit: int = 10,
    ) -> list[Item]:
        ns = self._ns(namespace_prefix)
        if query:
            memories = self._client.recall(query, namespace=ns, limit=limit)
        else:
            memories = self._client.memories(namespace=ns, limit=limit)

        items = []
        for m in memories:
            try:
                value = json.loads(m.content)
            except (json.JSONDecodeError, TypeError):
                value = {"content": m.content}
            # Extract key from tags
            key = m.id
            for t in m.tags:
                if t.startswith("key:"):
                    key = t[4:]
                    break
            items.append(
                Item(
                    value=value,
                    key=key,
                    namespace=namespace_prefix,
                    created_at=m.created_at,
                    updated_at=m.updated_at,
                )
            )
        return items


__all__ = ["MrMemoryCheckpointer", "MrMemoryStore"]
