# MrMemory — Agent Memory Relay

[![PyPI](https://img.shields.io/pypi/v/mrmemory)](https://pypi.org/project/mrmemory/)
[![License](https://img.shields.io/badge/license-MIT-blue)](LICENSE)
[![Docs](https://img.shields.io/badge/docs-mrmemory.dev-8b7aff)](https://mrmemory.dev/docs)

Persistent long-term memory for AI agents. One line to install, three lines to integrate.

**[Docs](https://mrmemory.dev/docs)** · **[API Reference](https://mrmemory.dev/docs#endpoints)** · **[Website](https://mrmemory.dev)**

## Install

```bash
pip install mrmemory
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
| GET | `/v1/recall?q=...` | Semantic search |
| DELETE | `/v1/forget/:id` | Delete a memory |
| GET | `/v1/memories` | List all memories |

Auth: `Authorization: Bearer amr_sk_...`

## Pricing

Starts at **$5/mo** — 10K memories, 50K API calls. [Sign up →](https://amr-memory-api.fly.dev/v1/billing/checkout)

## Links

- **Docs:** https://mrmemory.dev/docs.html
- **Dashboard:** https://mrmemory.dev
- **GitHub:** https://github.com/amr-dev/amr-python

## License

MIT
