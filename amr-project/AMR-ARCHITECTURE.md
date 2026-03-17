# AMR Architecture — Agent Memory Relay

## Overview

AMR is a neutral, external long-term memory layer for AI agents. Any agent, any framework, any provider — one API to remember, recall, share, and forget. **$5/mo.**

## Design Principles

1. **Framework-agnostic** — Works with OpenClaw, LangChain, CrewAI, AutoGen, raw API calls
2. **Dead simple** — 3 lines to integrate, zero config beyond API key
3. **Fast** — p99 < 50ms for recall, p99 < 20ms for remember
4. **Secure** — Tenant-isolated, encrypted at rest, API-key scoped
5. **Affordable** — $5/mo covers 99% of use cases

---

## API Design

### Base URL
```
https://api.amr.dev/v1
```

### Authentication
Every request requires an API key in the header:
```
Authorization: Bearer amr_sk_...
```

JWTs are issued for dashboard sessions. API keys for agent access.

### Core Endpoints (REST)

#### POST /v1/remember
Store a memory.
```json
{
  "content": "User prefers dark mode and uses vim keybindings",
  "tags": ["preferences", "editor"],
  "namespace": "user-settings",
  "agent_id": "my-assistant",
  "ttl": null
}
```
Response: `201 Created` with memory ID.

The content is automatically embedded into a vector. Metadata is stored in PostgreSQL. The vector goes to Qdrant.

#### POST /v1/recall
Retrieve relevant memories via semantic search.
```json
{
  "query": "What editor does the user prefer?",
  "namespace": "user-settings",
  "agent_id": "my-assistant",
  "limit": 5,
  "threshold": 0.7
}
```
Response: Array of memories ranked by relevance, with similarity scores.

#### POST /v1/share
Share memories between agents or namespaces.
```json
{
  "memory_ids": ["mem_abc123"],
  "target_agent_id": "other-assistant",
  "permissions": "read"
}
```
Enables multi-agent collaboration with explicit permission grants.

#### DELETE /v1/forget
Remove memories permanently.
```json
{
  "memory_ids": ["mem_abc123"],
  "namespace": "user-settings"
}
```
Also supports: `{"forget_all": true, "namespace": "..."}` for bulk deletion.

#### GET /v1/memories
List memories with filtering.
```
GET /v1/memories?namespace=user-settings&agent_id=my-assistant&limit=20&offset=0
```

### WebSocket API

```
wss://api.amr.dev/v1/stream
```

Real-time memory events for agents that need live updates:
- `memory.created` — new memory stored
- `memory.shared` — memory shared with this agent
- `memory.expired` — TTL-expired memory removed

Use case: Agent B gets notified immediately when Agent A stores a relevant memory.

---

## Storage Layer

### PostgreSQL (Metadata)
```sql
CREATE TABLE memories (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id   UUID NOT NULL REFERENCES tenants(id),
    agent_id    TEXT NOT NULL,
    namespace   TEXT DEFAULT 'default',
    content     TEXT NOT NULL,
    tags        TEXT[] DEFAULT '{}',
    ttl         INTERVAL,
    expires_at  TIMESTAMPTZ,
    created_at  TIMESTAMPTZ DEFAULT now(),
    updated_at  TIMESTAMPTZ DEFAULT now()
);

CREATE TABLE tenants (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name        TEXT NOT NULL,
    email       TEXT NOT NULL UNIQUE,
    plan        TEXT DEFAULT 'starter',
    api_key     TEXT NOT NULL UNIQUE,
    created_at  TIMESTAMPTZ DEFAULT now()
);

CREATE TABLE memory_shares (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    memory_id       UUID REFERENCES memories(id) ON DELETE CASCADE,
    target_agent_id TEXT NOT NULL,
    permissions     TEXT DEFAULT 'read',
    created_at      TIMESTAMPTZ DEFAULT now()
);
```

Indexes: `(tenant_id, agent_id, namespace)`, `(tenant_id, tags)` using GIN, `(expires_at)` for TTL cleanup.

### Qdrant (Vectors)
- Collection per tenant (isolation)
- Vector dimension: 1536 (OpenAI ada-002) or 384 (all-MiniLM-L6-v2 for self-hosted)
- Payload: `{memory_id, agent_id, namespace, tags}`
- HNSW index with default params (tune under load)

### Embedding Pipeline
1. Memory content arrives via `/remember`
2. Text → embedding via configured model (default: OpenAI ada-002, option for self-hosted)
3. Vector stored in Qdrant, metadata in PostgreSQL
4. Both writes in a transaction-like flow (Qdrant write, then PG write; compensating delete on PG failure)

---

## Authentication & Multi-Tenancy

### API Keys
- Format: `amr_sk_` + 32 random bytes (base62)
- Hashed (SHA-256) in database, never stored plaintext
- Scoped to tenant — one key per tenant by default, multiple keys supported
- Key rotation: new key issued, old key valid for 24h overlap

### JWT (Dashboard)
- Issued on login (email magic link or OAuth)
- Short-lived (15min) with refresh tokens (7d)
- Used only for dashboard API, not agent API

### Tenant Isolation
- Every query filters by `tenant_id` — no cross-tenant data access
- Qdrant collections are per-tenant
- Rate limits are per-tenant
- API keys are tenant-scoped

---

## Pricing & Rate Limiting

### Starter — $5/mo
- 10,000 memories stored
- 50,000 API calls/month
- 5 agents
- 3 namespaces
- Community support
- Rate limit: 100 req/min

### Pro — $25/mo
- 100,000 memories
- 500,000 API calls/month
- Unlimited agents
- Unlimited namespaces
- WebSocket streaming
- Priority support
- Rate limit: 1,000 req/min

### Enterprise — Custom
- Unlimited everything
- Self-hosted option
- SLA
- Dedicated support
- Custom embedding models

### Rate Limiting Implementation
- Token bucket algorithm per API key
- Redis-backed counters
- Headers: `X-RateLimit-Limit`, `X-RateLimit-Remaining`, `X-RateLimit-Reset`
- 429 responses with `Retry-After` header

---

## SDK Design Philosophy

### Core Principles
1. **3-line quickstart** — import, init, use
2. **Zero config** — only API key required
3. **Idiomatic** — feels native in each language
4. **Type-safe** — full type coverage, autocomplete-friendly
5. **Async-first** — non-blocking by default (Python asyncio, TS async/await)

### Python Example
```python
from amr import AMR

amr = AMR("amr_sk_...")

# Remember
amr.remember("User prefers dark mode", tags=["prefs"])

# Recall
memories = amr.recall("What theme does the user like?")
```

### TypeScript Example
```typescript
import { AMR } from '@amr/client'

const amr = new AMR('amr_sk_...')

await amr.remember('User prefers dark mode', { tags: ['prefs'] })
const memories = await amr.recall('What theme does the user like?')
```

### Rust Example
```rust
use amr::Client;

let client = Client::new("amr_sk_...");
client.remember("User prefers dark mode").tags(&["prefs"]).send().await?;
let memories = client.recall("What theme does the user like?").send().await?;
```

---

## Infrastructure

### Deployment Target
- **Primary:** Fly.io (simple, good edge presence, affordable)
- **Database:** Neon (serverless PostgreSQL) or Supabase
- **Vectors:** Qdrant Cloud
- **Cache/Rate Limiting:** Upstash Redis
- **CDN:** Cloudflare (landing page + API edge caching for reads)

### Architecture Diagram (Text)
```
Client (SDK) → Cloudflare Edge → Fly.io (API Server - Rust)
                                      ├→ PostgreSQL (Neon) — metadata
                                      ├→ Qdrant Cloud — vectors
                                      ├→ Upstash Redis — rate limits, cache
                                      └→ Embedding API — OpenAI / self-hosted
```

### Health & Monitoring
- `/health` — basic liveness
- `/health/ready` — checks DB + Qdrant + Redis connectivity
- Prometheus metrics endpoint
- Grafana dashboards for latency, throughput, error rates
- PagerDuty/Opsgenie for alerting
