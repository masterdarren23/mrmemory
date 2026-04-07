# MrMemory — Operational Overview
**Prepared by Darren W, Solo Founder | March 2026**

---

## What MrMemory Is

MrMemory is a managed memory-as-a-service API for AI agents. It solves the problem that every AI agent forgets everything between sessions. One API call to store, one to recall — agents remember forever.

**Live product:** https://mrmemory.dev
**Live API:** https://amr-memory-api.fly.dev
**Source code:** https://github.com/masterdarren23/mrmemory

---

## Technology Stack

| Layer | Technology | Why |
|-------|-----------|-----|
| Backend API | Rust (Axum) | Sub-millisecond overhead, 30MB binary, memory-safe |
| Primary Database | PostgreSQL 17 | ACID guarantees, proven at scale, full-text + JSONB |
| Vector Search | Qdrant | Purpose-built for embeddings, cosine similarity, fast |
| Embeddings | OpenAI text-embedding-3-small | Best cost/quality ratio for semantic search |
| LLM (intelligence) | GPT-4o-mini (configurable) | Auto-extraction, compression, memory governance |
| Hosting | Fly.io (sjc region) | Edge deployment, 2 redundant machines, managed Postgres |
| Frontend | Static HTML/CSS on Vercel | Zero-cost hosting, auto-deploy from GitHub |
| Payments | Stripe | $5/mo subscription with 7-day free trial |

---

## Architecture

```
Client (Python/TypeScript SDK)
        │
        ▼
   Rust API (Axum) ──── PostgreSQL 17
        │                    │
        ├── OpenAI API       │ (memories, tenants,
        │   (embeddings +    │  api_keys, proposals,
        │    LLM calls)      │  namespace_policies)
        │                    │
        └── Qdrant ──────────┘
            (vector index,     (hydration from PG
             cosine search)     after vector match)
```

**Request flow:**
1. Client sends `remember("user prefers dark mode")` with API key
2. API validates auth, stores memory in PostgreSQL
3. Async: generates OpenAI embedding, upserts to Qdrant
4. Client sends `recall("what does the user prefer?")`
5. API embeds query → Qdrant cosine search → hydrate results from PG → return

**Typical latency:** ~18ms p50 for recall, ~12ms p50 for store

---

## Product Features

### Core
- **remember()** — store with auto-embedding and indexing
- **recall()** — semantic vector search with cosine similarity scores
- **share()** — multi-agent memory sharing with permissions

### Intelligent
- **auto_remember()** — LLM extracts structured memories from raw conversations
- **compress()** — group similar memories, LLM summarizes, reduce count by ~44%
- **Self-edit tools** — update(), merge(), delete_outdated() for agent-managed memory

### Governance (Three-Layer)
- **Core memories** — permanent shared truth, governed by LLM judge
- **Private memories** — agent scratchpad, 24h TTL, ungoverned
- **Provisional memories** — proposed by agents, auto-judged for promotion to core
- **Human override** — supervisor/human can approve/reject any proposal

### Infrastructure
- **Real-time sync** — WebSocket events for multi-agent memory sharing
- **Write-through validation** — LLM checks for contradictions before storing
- **Provenance tracking** — created_by, modified_by, confidence, write_source on every memory
- **Namespace policies** — open, append-only, or read-only per namespace
- **Per-tenant config** — configurable judge model, provisional TTL, memory limits

### Integrations
- **LangChain/LangGraph** — drop-in MrMemoryCheckpointer + MrMemoryStore
- **CrewAI** — direct integration example
- **OpenClaw** — native support
- **Docker Compose** — self-hosted option (Postgres + Qdrant + API)

---

## SDKs

| SDK | Package | Version | Install |
|-----|---------|---------|---------|
| Python | mrmemory (PyPI) | 0.4.1 | `pip install mrmemory` |
| TypeScript | memorymr (npm) | 0.4.1 | `npm install memorymr` |
| Rust | N/A (reqwest examples) | — | Direct HTTP |

---

## Deployment

- **2 Fly.io machines** (redundant, sjc region, rolling deploys)
- **Fly Postgres** (managed, automatic backups)
- **Fly Qdrant** (1GB volume, REST API)
- **Docker image:** 30MB (multi-stage Rust build)
- **Build time:** ~2 minutes (remote build on Fly)
- **Zero-downtime deploys** via rolling machine replacement

---

## Team

**Solo founder:** Darren W
- Built entire product (backend, SDKs, landing page, docs, billing) in March 2026
- Development assisted by a 6-agent AI team via OpenClaw:
  - Main agent (project lead), Backend (Rust), SDK (Python/TS), Frontend, Marketing, DevOps
- This is a demonstration of the product's own thesis: AI agents with persistent memory can build real products

---

## Traction (as of March 2026)

- **146 unique GitHub cloners** in first 6 hours after first Reddit post
- **795 views** on initial Reddit post (18% clone rate)
- **Organic-only distribution** — $0 marketing spend
- **Quality engagement** — production engineers asking about persistence guarantees, scoping, governance
- **Published SDKs** downloaded from PyPI and npm

---

## Competitive Landscape

| Feature | MrMemory | Mem0 ($24M funded) | Letta/MemGPT | Raw Qdrant |
|---------|----------|-------|------|------|
| Managed API | ✅ | ✅ | ❌ Self-host | ✅ |
| Auto-extraction | ✅ | ✅ | ✅ | ❌ |
| Compression | ✅ | ❌ | ❌ | ❌ |
| Self-edit tools | ✅ | ❌ | Partial | ❌ |
| Memory governance | ✅ | ❌ | ❌ | ❌ |
| Real-time WebSocket | ✅ | ❌ | ❌ | ❌ |
| LangGraph native | ✅ | ❌ | ❌ | ❌ |
| Rust backend | ✅ | ❌ | ❌ (Python) | ✅ |
| Self-hostable | ✅ | ❌ | ✅ | ✅ |

**Key differentiator:** Only memory API with governed memory promotion, compression, self-edit tools, and real-time multi-agent sync. Rust backend is 4x faster than Python alternatives.

---

## Contact

**Darren W** — Solo Founder
📞 +1 (586) 277-9419
🌐 mrmemory.dev
📦 github.com/masterdarren23/mrmemory
