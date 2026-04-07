"""Tests for MrMemory LangGraph integration.

These tests mock the AMR client to avoid network calls.
"""

from __future__ import annotations

import json
from datetime import datetime
from unittest.mock import MagicMock, patch

import pytest

from amr.types import Memory


def _make_memory(id: str, content: str, namespace: str = "default", tags: list[str] | None = None) -> Memory:
    return Memory(
        id=id,
        content=content,
        namespace=namespace,
        tags=tags or [],
        created_at=datetime(2026, 1, 1),
        updated_at=datetime(2026, 1, 1),
    )


# ---------------------------------------------------------------------------
# Import guard
# ---------------------------------------------------------------------------

def test_import_error_without_langgraph():
    """Importing langchain module should fail gracefully without langgraph."""
    # We can't easily test this since langgraph may or may not be installed.
    # Just verify the module loads when deps are present.
    try:
        from mrmemory.langchain import MrMemoryCheckpointer, MrMemoryStore
        assert MrMemoryCheckpointer is not None
        assert MrMemoryStore is not None
    except ImportError:
        pytest.skip("langgraph not installed")


# ---------------------------------------------------------------------------
# Checkpointer
# ---------------------------------------------------------------------------

@pytest.fixture
def checkpointer():
    try:
        from mrmemory.langchain import MrMemoryCheckpointer
    except ImportError:
        pytest.skip("langgraph not installed")

    with patch("mrmemory.langchain.AMR") as MockAMR:
        mock_client = MagicMock()
        MockAMR.return_value = mock_client
        cp = MrMemoryCheckpointer(api_key="amr_sk_test")
        cp._client = mock_client
        yield cp, mock_client


def test_checkpointer_put(checkpointer):
    cp, mock_client = checkpointer
    mock_client.remember.return_value = _make_memory("m1", "{}", namespace="langgraph:cp:t1")

    config = {"configurable": {"thread_id": "t1"}}
    checkpoint = {"id": "cp-123", "ts": "2026-01-01", "channel_values": {}, "channel_versions": {}, "versions_seen": {}}
    metadata = {"source": "input", "step": 0}

    result = cp.put(config, checkpoint, metadata)

    assert result["configurable"]["thread_id"] == "t1"
    assert result["configurable"]["checkpoint_id"] == "cp-123"
    mock_client.remember.assert_called_once()
    call_args = mock_client.remember.call_args
    assert call_args.kwargs["namespace"] == "langgraph:cp:t1"


def test_checkpointer_get_tuple(checkpointer):
    cp, mock_client = checkpointer

    checkpoint = {"id": "cp-123", "ts": "2026-01-01", "channel_values": {}, "channel_versions": {}, "versions_seen": {}}
    payload = json.dumps({"checkpoint": checkpoint, "metadata": {"step": 0}, "parent_checkpoint_id": None, "checkpoint_ns": ""})
    mock_client.memories.return_value = [_make_memory("m1", payload, namespace="langgraph:cp:t1")]

    config = {"configurable": {"thread_id": "t1"}}
    result = cp.get_tuple(config)

    assert result is not None
    assert result.checkpoint["id"] == "cp-123"


def test_checkpointer_get_tuple_empty(checkpointer):
    cp, mock_client = checkpointer
    mock_client.memories.return_value = []

    config = {"configurable": {"thread_id": "t1"}}
    assert cp.get_tuple(config) is None


def test_checkpointer_list(checkpointer):
    cp, mock_client = checkpointer

    checkpoint = {"id": "cp-1", "ts": "2026-01-01", "channel_values": {}, "channel_versions": {}, "versions_seen": {}}
    payload = json.dumps({"checkpoint": checkpoint, "metadata": {}, "parent_checkpoint_id": None, "checkpoint_ns": ""})
    mock_client.memories.return_value = [_make_memory("m1", payload)]

    config = {"configurable": {"thread_id": "t1"}}
    results = list(cp.list(config, limit=5))
    assert len(results) == 1
    assert results[0].checkpoint["id"] == "cp-1"


# ---------------------------------------------------------------------------
# Store
# ---------------------------------------------------------------------------

@pytest.fixture
def store():
    try:
        from mrmemory.langchain import MrMemoryStore
    except ImportError:
        pytest.skip("langgraph not installed")

    with patch("mrmemory.langchain.AMR") as MockAMR:
        mock_client = MagicMock()
        MockAMR.return_value = mock_client
        s = MrMemoryStore(api_key="amr_sk_test")
        s._client = mock_client
        yield s, mock_client


def test_store_put_and_get(store):
    s, mock_client = store
    mock_client.memories.return_value = []
    mock_client.remember.return_value = _make_memory("m1", '{"value": "dark"}', tags=["key:theme"])

    s._put(("user", "prefs"), "theme", {"value": "dark"})
    mock_client.remember.assert_called_once()

    # Now test get
    mock_client.memories.return_value = [
        _make_memory("m1", '{"value": "dark"}', namespace="user:prefs", tags=["key:theme"])
    ]
    item = s._get(("user", "prefs"), "theme")
    assert item is not None
    assert item.value == {"value": "dark"}
    assert item.key == "theme"


def test_store_delete(store):
    s, mock_client = store
    mock_client.memories.return_value = [_make_memory("m1", '{}', tags=["key:theme"])]

    s._put(("user", "prefs"), "theme", None)
    mock_client.forget.assert_called_once_with("m1")


def test_store_search(store):
    s, mock_client = store
    mock_client.recall.return_value = [
        _make_memory("m1", '{"value": "dark"}', tags=["key:theme"])
    ]

    results = s._search(("user", "prefs"), "theme preference", limit=5)
    assert len(results) == 1
    assert results[0].key == "theme"
