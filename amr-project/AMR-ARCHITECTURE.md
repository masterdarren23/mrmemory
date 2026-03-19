# AMR Architecture — Agent Memory Relay

## Overview

AMR is a neutral, external long-term memory layer for AI agents. Any agent, any framework, any provider — one API to remember, recall, share, and forget. **$5/mo.**

## Design Principles

1. **Framework-agnostic** — Works with OpenClaw, LangChain, CrewAI, AutoGen, raw API calls
2. **Dead simple** — 3 lines to integrate, zero config beyond API key
3. **Fast** — Sub-second semantic recall with real vector search
4. **Secure** — Tenant-isolated, encrypted at rest, API-key scoped
5. **Affordable** — $5/mo covers 99% of use cases

---

## API Design

### Base URL
```
https://amr-memory-api.fly.dev/v1
```

### Authentication
Every request requires an API key in the header:
```
Authorization: Bearer amr_sk_...
```

### Core Endpoints (REST)

#### POST /v1/memories
Store a memory. Content is automatically embedded into a vector (OpenAI ada-002) and indexed in Qdrant.
```json
{
  "content": "User prefers dark mode and uses vim keybindings",
  "tags": ["preferences", "editor"],
  "namespace": "user-settings",
  "agent_id": "my-assistant"
}
```
Response: `201 Created` with full memory object.

#### GET /v1/memories/recall
Retrieve relevant memories via semantic search (OpenAI embeddings → Qdrant cosine similarity → Postgres hydration).
```
GET /v1/memories/recall?query=What+editor+does+the+user+prefer&namespace=user-settings&limit=5&threshold=0.7
```
Response: Array of memories ranked by relevance with real similarity scores.

#### GET /v1/memories
List memories with filtering and pagination.
```
GET /v1/memories?namespace=user-settings&agent_id=my-assistant&limit=20&offset=0
```

#### DELETE /v1/memories/{id}
Remove a specific memory permanently.
```
DELETE /v1/memories/mem_abc123
```
Response: `204 No Content`

#### POST /v1/auth/keys
Create a new API key (requires existing auth).
```json
{
  "name": "my-bot-key",
  "scopes": ["memories:read", "memories:write", "memories:delete"]
}
```
Response: API key (shown only once).

#### POST /v1/billing/webhook
Stripe webhook endpoint for payment processing.

#### GET /v1/welcome?session_id=...
Post-payment welcome page with auto-provisioned API key.

### SDK Aliases
The SDKs use friendlier method names that map to these endpoints:
- `client.remember()` → `POST /v1/memories`
- `client.recall()` → `GET /v1/memories/recall`
- `client.forget()` → `DELETE /v1/memories/{id}`
- `client.list()` → `GET /v1/memories`

### WebSocket API

```
wss://amr-memory-api.fly.dev/v1/ws?token=amr_sk_...
```

Real-time memory events for multi-agent collaboration:

**Client → Server:**
```json
{"type": "subscribe", "namespace": "weather-trading", "agent_id": "bot-1"}
{"type": "unsubscribe", "namespace": "weather-trading"}
{"type": "presence"}
{"type": "ping"}
```

**Server → Client:**
```json
{"type": "memory.created", "memory": {...}, "namespace": "...", "agent_id": "..."}
{"type": "memory.deleted", "memory_id": "...", "namespace": "..."}
{"type": "presence.update", "agents": ["bot-1", "bot-2"]}
{"type": "pong"}
```

ACL-controlled: agents only receive events they have permission to see via the `memory_shares` table.

---

## Storage Layer

### PostgreSQL (Metadata)
Hosted on Fly.io (app: `amr-db`), database: `amr_memory_api`.

```sql
-- Core tables
tenants (id, external_id, name, email, plan, stripe_customer_id, stripe_session_id, created_at)
memories (id, external_id, tenant_id, agent_id, namespace, content, tags, metadata, expires_at, created_at, updated_at)
api_keys (id, external_id, tenant_id, name, key_hash, key_prefix, scopes, revoked_at, expires_at, last_used_at, created_at)
memory_shares (id, tenant_id, agent_id, target_agent_id, namespace, permission, created_at)
```

### Qdrant (Vectors)
Hosted on Fly.io (app: `amr-qdrant`). Single `memories` collection.
- Vector dimension: 1536 (OpenAI text-embedding-3-small)
- Distance: Cosine
- Payload: `{tenant_id, external_id, namespace, agent_id}`
- Filtered by `tenant_id` on every search for tenant isolation

### Embedding Pipeline
1. Memory content arrives via `POST /v1/memories`
2. Text → embedding via OpenAI text-embedding-3-small (async, best-effort)
3. Vector + payload upserted to Qdrant
4. Metadata stored in PostgreSQL
5. On recall: query → embedding → Qdrant cosine search → Postgres hydration

---

## Authentication & Multi-Tenancy

### API Keys
- Format: `amr_sk_` + 64 base62 chars (32 random bytes)
- SHA-256 hashed in database, never stored plaintext
- Scoped to tenant — multiple keys per tenant supported
- `last_used_at` tracked on every request

### Tenant Isolation
- Every SQL query filters by `tenant_id`
- Qdrant searches filter by `tenant_id` in payload
- WebSocket events scoped by tenant + ACL permissions
- No cross-tenant data access possible

---

## Pricing

### Starter — $5/mo
- 10,000 memories stored
- 50,000 API calls/month
- 5 agents
- Real-time WebSocket events
- Semantic vector search
- Auto-embedding (you send text, we handle vectors)
- Tenant-isolated, encrypted at rest

Payment: [Stripe Payment Link](https://buy.stripe.com/9B6eVf0Jw0GefiP36l8g000)

---

## SDK Design

### Python
```python
from mrmemory import AMR

client = AMR("amr_sk_...", agent_id="trading-bot", namespace="weather-trading")
client.remember("Customer wants a weather prediction model", tags=["weather"])
results = client.recall("What does this customer want?")
```
Install: `pip install mrmemory`

### TypeScript
```typescript
import { AMR } from 'memorymr'

const client = new AMR({ apiKey: 'amr_sk_...' })
await client.remember('Customer wants a weather prediction model', { tags: ['weather'] })
const results = await client.recall('What does this customer want?')
```
Install: `npm install memorymr`

---

## Infrastructure

### Current Deployment
```
Client (SDK) → Fly.io (amr-memory-api, Rust/Axum, sjc region)
                    ├→ PostgreSQL (Fly.io amr-db) — metadata + auth
                    ├→ Qdrant (Fly.io amr-qdrant) — vector search
                    └→ OpenAI API — embeddings (text-embedding-3-small)
```

Landing page: [mrmemory.dev](https://mrmemory.dev) (Vercel)
Docs: [mrmemory.dev/docs](https://mrmemory.dev/docs)
GitHub: [github.com/masterdarren23/mrmemory](https://github.com/masterdarren23/mrmemory)

### Health & Monitoring
- `GET /health` — liveness check (always responds if server is up)
- Fly.io built-in monitoring and log aggregation

