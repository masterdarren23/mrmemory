# 🧠 MrMemory — Agent Memory Relay

**Long-term memory for AI agents. Install in one line. Remember forever.**

```bash
pip install mrmemory
```

```python
from mrmemory import AMR

amr = AMR("amr_sk_...")
amr.remember("User prefers dark mode and vim keybindings")

# Later, in any session...
memories = amr.recall("What are the user's preferences?")
# → "User prefers dark mode and vim keybindings" (score: 0.94)
```

## What is AMR?

AMR is a hosted memory layer for AI agents. It handles:

- **Semantic storage** — auto-embedding via OpenAI, indexed in Qdrant
- **Semantic recall** — real cosine similarity search, not keyword matching
- **Multi-agent sharing** — real-time WebSocket events with ACL permissions
- **Tenant isolation** — every query scoped by tenant_id, no cross-tenant access
- **Dead simple** — `pip install mrmemory` and three lines of code

You call `remember()` and `recall()`. We handle everything else.

## Architecture

```
┌─────────────┐     ┌──────────────┐     ┌──────────┐
│  Your Agent  │────▶│  AMR API     │────▶│ Qdrant   │
│  (Python/TS) │     │  (Rust/Axum) │     │ (vectors)│
└─────────────┘     └──────┬───────┘     └──────────┘
                           │
                    ┌──────┴───────┐
                    │  PostgreSQL  │
                    │  (metadata)  │
                    └──────────────┘
```

- **API**: Rust (Axum) — fast, safe, no GC pauses
- **Vector DB**: Qdrant — cosine similarity search (1536-dim OpenAI embeddings)
- **Metadata**: PostgreSQL — tenants, API keys, memories
- **Real-time**: WebSocket push for multi-agent memory sharing

## SDKs

| Language | Package | Status |
|----------|---------|--------|
| Python | `pip install mrmemory` | ✅ Ready |
| TypeScript | `npm install memorymr` | ✅ Ready |
| Rust | `cargo add amr` | 🔜 Planned |

### Framework Integrations

- LangChain
- CrewAI
- AutoGen
- OpenClaw

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/v1/memories` | Store a memory |
| `GET` | `/v1/memories` | List memories |
| `GET` | `/v1/memories/recall` | Semantic search |
| `DELETE` | `/v1/memories/{id}` | Delete a memory |
| `POST` | `/v1/auth/keys` | Generate API key |
| `GET` | `/health` | Health check |
| `GET` | `/health/ready` | Readiness check |

## Pricing

| Plan | Price | Memories | API Calls | Agents |
|------|-------|----------|-----------|--------|
| Starter | $5/mo | 10,000 | 50,000/mo | 5 |
| Pro | $25/mo | 100,000 | 500,000/mo | Unlimited |
| Enterprise | Custom | Custom | Custom | Custom |

Free during beta.

## Development

```bash
# Start local infrastructure
docker compose up -d

# Run the API
cd api && cargo run

# Run Python SDK tests
cd sdk-python/python-sdk && pip install -e ".[dev]" && pytest

# Run TypeScript SDK tests
cd sdk-typescript && npm install && npm test
```

## Project Structure

```
amr-project/
├── api/                  # Rust API server (Axum)
├── sdk-python/           # Python SDK
├── sdk-typescript/       # TypeScript SDK
├── landing-page/         # mrmemory.dev (Vercel)
├── infra/                # Docker, CI/CD, deployment
│   ├── Dockerfile
│   ├── docker-compose.yml
│   └── .github/workflows/ci.yml
└── docs/                 # Documentation
```

## License

MIT

---

**Website**: [mrmemory.dev](https://mrmemory.dev)

