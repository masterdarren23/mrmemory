[README.md](https://github.com/user-attachments/files/26316854/README.md)
# 🧠 MrMemory — Agent Memory Relay

[![PyPI](https://img.shields.io/pypi/v/mrmemory)](https://pypi.org/project/mrmemory/)
[![npm](https://img.shields.io/npm/v/memorymr)](https://www.npmjs.com/package/memorymr)
[![License](https://img.shields.io/badge/license-MIT-blue)](LICENSE)
[![Docs](https://img.shields.io/badge/docs-mrmemory.dev-8b7aff)](https://mrmemory.dev/docs)

**Install in one line. Remember forever.** MrMemory is the memory layer for AI agents — works with any framework. Starts at $5/mo.

```bash
pip install mrmemory        # Python
npm install memorymr         # TypeScript
```

## Quickstart

```python
from mrmemory import AMR

amr = AMR("amr_sk_...")
amr.remember("User prefers dark mode and vim keybindings")

memories = amr.recall("What are the user's preferences?")
# → "User prefers dark mode and vim keybindings" (score: 0.94)
```

## Features

| Feature | Description | API |
|---------|-------------|-----|
| 🧠 **Remember** | Store memories with auto-embedding | `remember()` |
| 🔍 **Recall** | Semantic vector search | `recall()` |
| ✨ **Auto-Remember** | LLM extracts memories from conversations | `auto_remember()` |
| 🗜️ **Compress** | Merge similar memories into denser ones | `compress()` |
| ✏️ **Self-Edit** | Update, merge, and prune memories | `update()` / `merge()` / `delete_outdated()` |
| 🔗 **LangChain Native** | Drop-in checkpointer + store for LangGraph | `MrMemoryCheckpointer` |
| 🤝 **Multi-Agent** | Share memories between agents with ACL | `share()` |
| ⚡ **Real-Time** | WebSocket push for memory events | `/v1/ws` |

## Auto-Remember

Extract memories from conversations automatically using GPT-4o-mini:

```python
result = amr.auto_remember([
    {"role": "user", "content": "I love hiking and my favorite language is Rust"},
    {"role": "assistant", "content": "Great choices!"},
], sync=True)
# {"extracted": 2, "created": 2, "duplicates_skipped": 0}
```

## Self-Edit Tools

Agents manage their own memory — MemGPT-style memory control:

```python
# Update (re-embeds automatically)
amr.update("mem_abc123", content="User now prefers light mode", tags=["preference"])

# Bulk delete old memories
result = amr.delete_outdated(older_than_seconds=86400 * 30, dry_run=True)
print(f"Would delete {result['deleted']} memories")

# Merge memories (LLM summarization)
merged = amr.merge(["mem_abc123", "mem_def456", "mem_ghi789"])
print(merged.content)  # Summarized by GPT-4o-mini
```

Merged/compressed memories return `is_compressed: true` with `merged_from` tracking.

## Compress

Reduce memory bloat — groups semantically similar memories and merges them:

```python
result = amr.compress(namespace="default", sync=True, dry_run=True)
print(f"Would compress {result['groups_compressed']} groups")

result = amr.compress(namespace="default", sync=True)
print(f"Reduced {result['before_count']} → {result['after_count']} memories")
```

## LangChain / LangGraph

```python
from mrmemory.langchain import MrMemoryCheckpointer, MrMemoryStore
from langgraph.graph import StateGraph

checkpointer = MrMemoryCheckpointer(api_key="amr_sk_...")
store = MrMemoryStore(api_key="amr_sk_...")
graph = StateGraph(...).compile(checkpointer=checkpointer, store=store)
```

## CrewAI

```python
from crewai import Agent, Task, Crew
from mrmemory import AMR

memory = AMR("amr_sk_...", agent_id="researcher", namespace="research")
memory.remember("Project deadline is March 30th", tags=["project", "deadline"])

# Recall context for your agent
context = memory.recall("project deadlines", limit=5)
```

## TypeScript

```typescript
import { AMR } from 'memorymr'

const amr = new AMR('amr_sk_...')
await amr.remember('User prefers dark mode')
const memories = await amr.recall('preferences')

// Self-edit
await amr.update('mem_abc123', { content: 'Updated', tags: ['pref'] })
await amr.deleteOutdated({ olderThanSeconds: 86400 * 30 })
const merged = await amr.merge(['mem_abc123', 'mem_def456'])
```

## Self-Hosting

Run MrMemory on your own infrastructure:

```bash
git clone https://github.com/masterdarren23/mrmemory.git
cd mrmemory
cp .env.example .env   # Add your OPENAI_API_KEY
docker compose up -d   # API at http://localhost:8080
```

### Architecture

```
┌─────────────┐     ┌──────────────┐     ┌─────────────┐
│  Your Agent  │────▶│  MrMemory    │────▶│  Qdrant     │
│  (SDK/REST)  │     │  API (Rust)  │     │  (Vectors)  │
└─────────────┘     └──────┬───────┘     └─────────────┘
                           │
                    ┌──────▼───────┐
                    │  PostgreSQL  │
                    │  (Metadata)  │
                    └──────────────┘
```

- **API**: Rust/Axum — fast, safe, 30MB Docker image
- **Vectors**: Qdrant — cosine similarity (OpenAI embeddings)
- **Metadata**: PostgreSQL 17 — tenants, keys, memories
- **Real-time**: WebSocket push for multi-agent sharing

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/v1/memories` | Store a memory |
| `GET` | `/v1/memories` | List memories |
| `GET` | `/v1/memories/recall` | Semantic search |
| `PATCH` | `/v1/memories/{id}` | Update a memory |
| `DELETE` | `/v1/memories/{id}` | Delete a memory |
| `POST` | `/v1/memories/auto` | LLM auto-remember |
| `POST` | `/v1/memories/compress` | Compress memories |
| `DELETE` | `/v1/memories/outdated` | Bulk delete by age/tags |
| `POST` | `/v1/memories/merge` | Merge memories |
| `GET` | `/v1/stats` | Usage & observability stats |
| `POST` | `/v1/auth/keys` | Generate API key |
| `GET` | `/v1/ws` | WebSocket events |

Auth: `Authorization: Bearer amr_sk_...`

## SDKs

| Language | Package | Version |
|----------|---------|---------|
| Python | [`mrmemory`](https://pypi.org/project/mrmemory/) | 0.4.1 |
| TypeScript | [`memorymr`](https://www.npmjs.com/package/memorymr) | 0.4.1 |
| Rust | Direct REST API | — |

## Pricing

| Plan | Price | Memories | API Calls/mo |
|------|-------|----------|--------------|
| **Starter** | $5/mo | 10,000 | 50,000 |
| **Pro** | $25/mo | 100,000 | 500,000 |
| **Self-Host** | Free | Unlimited | Unlimited |

[**Start Free → $5/mo**](https://buy.stripe.com/9B6eVf0Jw0GefiP36l8g000)

## Project Structure

```
mrmemory/
├── api/                    # Rust API server (Axum)
│   ├── src/
│   ├── migrations/
│   └── Dockerfile
├── sdk-python/             # Python SDK (mrmemory)
├── sdk-typescript/         # TypeScript SDK (memorymr)
├── landing/                # mrmemory.dev (Vercel)
│   └── docs/
├── docker-compose.yml      # Self-hosting stack
└── .env.example
```

## Links

- 🌐 **Website**: [mrmemory.dev](https://mrmemory.dev)
- 📚 **Docs**: [mrmemory.dev/docs](https://mrmemory.dev/docs)
- 🐍 **PyPI**: [mrmemory](https://pypi.org/project/mrmemory/)
- 📦 **npm**: [memorymr](https://www.npmjs.com/package/memorymr)

## License

MIT
