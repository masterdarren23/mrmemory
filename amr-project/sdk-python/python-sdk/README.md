# MrMemory — Agent Memory Relay

[![PyPI](https://img.shields.io/pypi/v/mrmemory)](https://pypi.org/project/mrmemory/)
[![License](https://img.shields.io/badge/license-MIT-blue)](LICENSE)
[![Docs](https://img.shields.io/badge/docs-mrmemory.dev-8b7aff)](https://mrmemory.dev/docs)

Persistent long-term memory for AI agents. One line to install, three lines to integrate.

**[Docs](https://mrmemory.dev/docs)** · **[API Reference](https://mrmemory.dev/docs#endpoints)** · **[Website](https://mrmemory.dev)**

## Install

```bash
pip install mrmemory

# With LangChain/LangGraph support:
pip install mrmemory[langchain]
```

## Quickstart

```python
from amr import AMR

amr = AMR("amr_sk_...")  # or set AMR_API_KEY env var

# Store a memory
amr.remember("User prefers dark mode and vim keybindings")

# Semantic recall
memories = amr.recall("What are the user's preferences?")
for m in memories:
    print(m.content, m.score)

# Forget a memory
amr.forget(memories[0].id)
```

## LLM Auto-Remember

Extract memories from conversations automatically using GPT-4o-mini:

```python
# Extract and store memories from a conversation
result = amr.auto_remember([
    {"role": "user", "content": "I love hiking and my favorite language is Rust"},
    {"role": "assistant", "content": "Great choices!"},
], sync=True)

print(result)  # {"extracted": 2, "created": 2, "duplicates_skipped": 0, ...}
```

Supports async mode (fire-and-forget), deduplication, and BYOK (bring your own OpenAI key).

## Memory Compression

Compress related memories into denser representations:

```python
# Compress memories in a namespace (dry run first)
result = amr.compress(namespace="default", sync=True, dry_run=True)
print(f"Would compress {result['groups_compressed']} groups")

# Actually compress
result = amr.compress(namespace="default", sync=True)
print(f"Reduced {result['before_count']} → {result['after_count']} memories")
```

## LangChain / LangGraph Integration

Drop-in checkpointer and store for LangGraph:

```python
from mrmemory.langchain import MrMemoryCheckpointer, MrMemoryStore
from langgraph.graph import StateGraph

checkpointer = MrMemoryCheckpointer(api_key="amr_sk_...")
store = MrMemoryStore(api_key="amr_sk_...")

graph = StateGraph(...).compile(checkpointer=checkpointer, store=store)
```

## Async Support

```python
from amr import AsyncAMR

async with AsyncAMR("amr_sk_...") as amr:
    await amr.remember("User prefers dark mode")
    memories = await amr.recall("What does the user prefer?")
```

## Configuration

```python
amr = AMR(
    api_key="amr_sk_...",       # or set AMR_API_KEY env var
    agent_id="my-assistant",    # default agent ID
    namespace="default",        # default namespace
    timeout=10.0,               # seconds
    max_retries=3,              # retry on transient failures
)
```

## API Endpoints

All requests go to `https://amr-memory-api.fly.dev`.

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/v1/remember` | Store a memory |
| POST | `/v1/recall` | Semantic search |
| DELETE | `/v1/forget/:id` | Delete a memory |
| GET | `/v1/memories` | List all memories |
| POST | `/v1/memories/auto` | LLM auto-remember from conversations |
| POST | `/v1/memories/compress` | Compress related memories |
| GET | `/v1/ws` | WebSocket real-time events |

Auth: `Authorization: Bearer amr_sk_...`

## Pricing

Starts at **$5/mo** — 10K memories, 50K API calls. [Sign up →](https://amr-memory-api.fly.dev/v1/billing/checkout)

## Links

- **Docs:** https://mrmemory.dev/docs
- **Dashboard:** https://mrmemory.dev
- **GitHub:** https://github.com/masterdarren23/mrmemory

## License

MIT
