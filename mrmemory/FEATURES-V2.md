# AMR V2 Features — Roadmap

Priority order for shipping:

---

## Feature 1: LangChain/LangGraph Checkpointer + Store (PRIORITY: SHIP FIRST)
**Effort:** Low-Medium (mostly Python SDK work)
**Why:** Biggest distribution win — drop-in for the entire LangChain ecosystem

### Deliverables
- `MrMemoryCheckpointer(BaseCheckpointSaver)` — saves/loads LangGraph state to AMR
- `MrMemoryStore(BaseStore)` — implements LangGraph Store interface for cross-thread memory
- Pre-built nodes: `trim_messages`, `summarize_conversation` (call /v1/memories/auto when available)
- One-liner: `checkpointer=MrMemoryCheckpointer(api_key="amr_sk_...")`

### Implementation
- Lives in Python SDK (`mrmemory` package) as `mrmemory.langchain` submodule
- Maps LangGraph's `put/get/list` to our existing CRUD + recall API
- Namespaces map to LangGraph thread_ids
- Dependencies: `langgraph`, `langchain-core` as optional deps (`pip install mrmemory[langchain]`)
- No backend changes needed — existing API supports everything

### Backend endpoints used
- POST /v1/memories (put checkpoint)
- GET /v1/memories (list checkpoints)
- POST /v1/memories/recall (get by thread)
- DELETE /v1/memories/:id (cleanup)

---

## Feature 2: LLM-Orchestrated Auto-Remember (PRIORITY: SECOND)
**Effort:** Medium
**Why:** Core intelligence layer — enables compression and entity extraction

### Deliverables
- `POST /v1/memories/auto` — accepts raw conversation messages
- Server-side LLM: extract entities, facts, preferences, decisions
- Dedup against existing memories (semantic similarity before insert)
- Auto-assign TTL based on importance
- Returns what was stored

### Key Decisions (confirmed by Dirwood)
- **Cost model:** BYOK default (user provides OpenAI/Anthropic key in settings). Starter tier: 500 free auto_remember calls/mo using GPT-4o-mini. Pro: unlimited managed + higher rate limits.
- **Model:** GPT-4o-mini for v1. BYOK users can pass `llm_model` in payload.
- **Sync vs Async:** Async fire-and-forget + WebSocket callback. Return `{job_id, status: "processing"}`, push `memory.auto_completed` via WS. Optional `?sync=true` flag for <2s blocking.

### Backend Changes
- New endpoint: POST /v1/memories/auto
- New table: `auto_remember_jobs` (job_id, tenant_id, status, result, created_at)
- LLM client abstraction (OpenAI + Anthropic + BYOK)
- Tenant settings: `llm_api_key`, `llm_model` columns or separate `tenant_settings` table
- WS event: `memory.auto_completed`
- Rate limiting: 500/mo for starter, unlimited for pro

### LLM Prompt (extraction)
Given conversation messages, extract:
- Key facts (atomic, deduplicated)
- Entities (people, tools, preferences)
- Relationships between entities
- Suggested TTL (ephemeral=24h, important=30d, permanent=none)

---

## Feature 3: Compression + Summarization Engine (PRIORITY: THIRD)
**Effort:** Medium (builds on Feature 2's LLM pipeline)
**Why:** Directly competes on cost/latency with Mem0's 90% token reduction claims

### Deliverables
- `?compress=true` flag on remember/auto endpoints
- Server-side LLM summarizes long memories into atomic facts
- Versioned history: original kept, compressed version is primary
- Background compressor (runs on prune cycle or nightly)
- Observability: token savings metrics in recall responses
- Dashboard endpoint: GET /v1/usage/compression

### Implementation
- Reuses Feature 2's LLM pipeline
- New columns: `compressed_content`, `original_content`, `compression_ratio`, `version`
- Recall responses include `"compressed": true, "tokens_saved": 847`
- Background task alongside TTL pruning (tokio::spawn)
- Pro tier: unlimited compressed storage

---

## Feature 4: Graph + Hybrid Retrieval (PRIORITY: DEFERRED)
**Effort:** High
**Why:** Handles complex relationships flat vectors miss

### Pragmatic Path
- **v1 (ship with Feature 2):** `entities` and `entity_relations` tables in Postgres. Auto-extract on remember(). Query: `/v1/memories/recall?entity=X&relation=Y`. Recursive CTEs for graph-like queries.
- **v2 (later):** Apache AGE Postgres extension if query patterns demand it.
- **v3 (much later):** Dedicated graph DB only if Postgres can't keep up.

### Postgres Schema (v1)
```sql
CREATE TABLE entities (
    id UUID PRIMARY KEY,
    tenant_id UUID REFERENCES tenants(id),
    name TEXT NOT NULL,
    entity_type TEXT, -- person, tool, concept, preference
    metadata JSONB,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE entity_relations (
    id UUID PRIMARY KEY,
    tenant_id UUID REFERENCES tenants(id),
    source_entity_id UUID REFERENCES entities(id),
    target_entity_id UUID REFERENCES entities(id),
    relation_type TEXT NOT NULL, -- prefers, uses, caused_by, related_to
    memory_id UUID REFERENCES memories(id), -- source memory
    confidence FLOAT DEFAULT 1.0,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
```
