"""Tests for the async AMR client."""

import httpx
import pytest
import respx

from amr import AsyncAMR, Memory


API_KEY = "amr_sk_test_key_1234567890"
BASE_URL = "https://api.amr.dev/v1"

MOCK_MEMORY = {
    "id": "mem_abc123",
    "content": "User prefers dark mode",
    "tags": ["preferences"],
    "namespace": "default",
    "agent_id": "test-agent",
    "created_at": "2026-03-12T10:00:00Z",
    "updated_at": "2026-03-12T10:00:00Z",
}


@respx.mock
async def test_async_remember():
    respx.post(f"{BASE_URL}/remember").mock(
        return_value=httpx.Response(201, json=MOCK_MEMORY)
    )
    async with AsyncAMR(API_KEY) as amr:
        memory = await amr.remember("User prefers dark mode", tags=["preferences"])

    assert isinstance(memory, Memory)
    assert memory.id == "mem_abc123"


@respx.mock
async def test_async_recall():
    respx.post(f"{BASE_URL}/recall").mock(
        return_value=httpx.Response(200, json={"memories": [{**MOCK_MEMORY, "score": 0.92}]})
    )
    async with AsyncAMR(API_KEY) as amr:
        memories = await amr.recall("dark mode?")

    assert len(memories) == 1
    assert memories[0].score == 0.92


@respx.mock
async def test_async_share():
    respx.post(f"{BASE_URL}/share").mock(
        return_value=httpx.Response(200, json={"ok": True})
    )
    async with AsyncAMR(API_KEY) as amr:
        await amr.share("mem_abc123", target_agent="agent-2")


@respx.mock
async def test_async_forget():
    respx.delete(f"{BASE_URL}/forget").mock(
        return_value=httpx.Response(204)
    )
    async with AsyncAMR(API_KEY) as amr:
        await amr.forget("mem_abc123")


@respx.mock
async def test_async_memories():
    respx.get(f"{BASE_URL}/memories").mock(
        return_value=httpx.Response(200, json={"memories": [MOCK_MEMORY]})
    )
    async with AsyncAMR(API_KEY) as amr:
        memories = await amr.memories()

    assert len(memories) == 1
