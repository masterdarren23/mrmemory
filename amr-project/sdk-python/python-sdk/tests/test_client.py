"""Tests for the synchronous AMR client."""

import httpx
import pytest
import respx

from amr import AMR, Memory
from amr.errors import AuthenticationError, RateLimitError, NotFoundError, ValidationError


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
def test_remember():
    respx.post(f"{BASE_URL}/remember").mock(
        return_value=httpx.Response(201, json=MOCK_MEMORY)
    )
    with AMR(API_KEY) as amr:
        memory = amr.remember("User prefers dark mode", tags=["preferences"])

    assert isinstance(memory, Memory)
    assert memory.id == "mem_abc123"
    assert memory.content == "User prefers dark mode"
    assert memory.tags == ["preferences"]


@respx.mock
def test_recall():
    respx.post(f"{BASE_URL}/recall").mock(
        return_value=httpx.Response(200, json={"memories": [{**MOCK_MEMORY, "score": 0.95}]})
    )
    with AMR(API_KEY) as amr:
        memories = amr.recall("What does the user prefer?")

    assert len(memories) == 1
    assert memories[0].score == 0.95
    assert memories[0].content == "User prefers dark mode"


@respx.mock
def test_share():
    respx.post(f"{BASE_URL}/share").mock(
        return_value=httpx.Response(200, json={"ok": True})
    )
    with AMR(API_KEY) as amr:
        amr.share("mem_abc123", target_agent="agent-2")  # no exception = success


@respx.mock
def test_forget():
    respx.delete(f"{BASE_URL}/forget").mock(
        return_value=httpx.Response(204)
    )
    with AMR(API_KEY) as amr:
        amr.forget("mem_abc123")  # no exception = success


@respx.mock
def test_forget_all():
    respx.delete(f"{BASE_URL}/forget").mock(
        return_value=httpx.Response(204)
    )
    with AMR(API_KEY) as amr:
        amr.forget_all(namespace="test")


@respx.mock
def test_memories_list():
    respx.get(f"{BASE_URL}/memories").mock(
        return_value=httpx.Response(200, json={"memories": [MOCK_MEMORY]})
    )
    with AMR(API_KEY) as amr:
        memories = amr.memories(limit=10)

    assert len(memories) == 1
    assert memories[0].id == "mem_abc123"


@respx.mock
def test_auth_error():
    respx.post(f"{BASE_URL}/remember").mock(
        return_value=httpx.Response(401, json={"error": "Invalid API key"})
    )
    with AMR(API_KEY, max_retries=0) as amr:
        with pytest.raises(AuthenticationError):
            amr.remember("test")


@respx.mock
def test_rate_limit_error():
    respx.post(f"{BASE_URL}/remember").mock(
        return_value=httpx.Response(429, json={"error": "Too many requests"}, headers={"retry-after": "5"})
    )
    with AMR(API_KEY, max_retries=0) as amr:
        with pytest.raises(RateLimitError) as exc_info:
            amr.remember("test")
        assert exc_info.value.retry_after == 5.0


@respx.mock
def test_not_found_error():
    respx.delete(f"{BASE_URL}/forget").mock(
        return_value=httpx.Response(404, json={"error": "Memory not found"})
    )
    with AMR(API_KEY, max_retries=0) as amr:
        with pytest.raises(NotFoundError):
            amr.forget("mem_nonexistent")


@respx.mock
def test_validation_error():
    respx.post(f"{BASE_URL}/remember").mock(
        return_value=httpx.Response(422, json={"error": "content is required"})
    )
    with AMR(API_KEY, max_retries=0) as amr:
        with pytest.raises(ValidationError):
            amr.remember("")


def test_no_api_key():
    import os
    env_backup = os.environ.pop("AMR_API_KEY", None)
    try:
        with pytest.raises(ValueError, match="No API key"):
            AMR()
    finally:
        if env_backup:
            os.environ["AMR_API_KEY"] = env_backup


def test_env_api_key():
    import os
    os.environ["AMR_API_KEY"] = "amr_sk_from_env"
    try:
        amr = AMR()
        assert amr._config.api_key == "amr_sk_from_env"
        amr.close()
    finally:
        del os.environ["AMR_API_KEY"]
