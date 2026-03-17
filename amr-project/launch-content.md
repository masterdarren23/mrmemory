# AMR Launch Content

## Show HN

**Title:** Show HN: AMR – Persistent memory for AI agents, one-line install, $5/mo

**Body:**

Hey HN,

I built AMR (Agent Memory Relay) because every AI agent I worked with forgot everything between sessions. I got tired of stuffing context into system prompts or rolling my own vector DB pipeline.

AMR is a hosted memory API. You install the SDK (`pip install mrmemory` or `npm install memorymr`), add your API key, and call `amr.remember()` / `amr.recall()`. That's it. We handle embeddings, vector storage, tenant isolation, and sub-50ms semantic recall.

**How it works:**
- `POST /v1/remember` — store a memory (we embed it automatically)
- `GET /v1/recall?q=...` — semantic search across your memories
- `DELETE /v1/forget/:id` — delete a memory
- `GET /v1/memories` — list everything

It works with any framework — LangChain, CrewAI, AutoGen, raw OpenAI, whatever. It's just an API.

$5/mo for 10K memories and 50K API calls. No usage-based surprises.

Built with Rust (Axum), Qdrant for vectors, PostgreSQL for metadata. Deployed on Fly.io.

Live API: https://amr-memory-api.fly.dev
Docs: https://mrmemory.dev/docs.html
Landing: https://mrmemory.dev

Would love feedback on the API design and pricing.

---

## Twitter/X Thread

**Tweet 1:**
Your AI agent forgets everything. Every. Single. Session.

I built AMR — persistent memory for AI agents in one line of code.

`pip install mrmemory`

$5/mo. Works with any framework. Ship it today →
https://mrmemory.dev

🧵👇

**Tweet 2:**
How it works:

```python
from amr import AMR
amr = AMR("amr_sk_...")

amr.remember("User prefers dark mode")
memories = amr.recall("What does the user like?")
# → "User prefers dark mode" (score: 0.94)
```

3 lines. That's the whole integration.

**Tweet 3:**
What you DON'T have to build:
- ❌ Embedding pipeline
- ❌ Vector database setup
- ❌ Retrieval logic
- ❌ Tenant isolation
- ❌ Cleanup/TTL management

AMR handles all of it. You call remember() and recall().

**Tweet 4:**
Works with everything:
- LangChain ✅
- CrewAI ✅
- AutoGen ✅
- OpenClaw ✅
- Raw OpenAI ✅
- Any HTTP client ✅

Python SDK, TypeScript SDK. REST API for everything else.

**Tweet 5:**
Pricing: $5/mo starter. 10K memories, 50K API calls.

No per-token billing. No usage-based surprises. Cancel anytime.

API is live right now: https://amr-memory-api.fly.dev
Docs: https://mrmemory.dev/docs.html

Give your agents a brain. 🧠

---

## Reddit — r/LocalLLaMA

**Title:** AMR: Persistent memory layer for AI agents — one-line install, works with any framework ($5/mo)

**Body:**

I've been building AI agents and the biggest pain point is always memory. Every session starts from zero. You either stuff everything into the prompt (expensive) or build your own vector DB pipeline (weeks of work).

So I built AMR (Agent Memory Relay). It's a hosted API that gives your agents persistent, semantic memory:

```python
pip install mrmemory

from amr import AMR
amr = AMR("amr_sk_...")
amr.remember("User prefers dark mode and local models")
memories = amr.recall("What does the user prefer?")
```

**Key points:**
- Works with ANY framework — LangChain, CrewAI, AutoGen, or plain API calls
- Semantic recall with sub-50ms latency
- You send text, we handle embeddings and vector storage
- Tenant-isolated, encrypted at rest
- $5/mo for 10K memories and 50K API calls
- Python and TypeScript SDKs

Works great with local LLM setups too — your model doesn't need to know about AMR, you just inject recalled memories into the prompt.

API: https://amr-memory-api.fly.dev
Docs: https://mrmemory.dev/docs.html
Site: https://mrmemory.dev

Happy to answer questions about the architecture (Rust + Qdrant + PostgreSQL).

---

## Reddit — r/LangChain

**Title:** Built a memory-as-a-service API that works with LangChain (and everything else) — $5/mo

**Body:**

LangChain's built-in memory works for simple cases, but when you need persistent memory across sessions, multi-agent sharing, or proper semantic recall — you end up building a lot of infrastructure.

AMR (Agent Memory Relay) is a hosted memory API that handles all of that:

```python
pip install mrmemory

from amr import AMR
amr = AMR("amr_sk_...")

# In your LangChain agent's tool or callback:
amr.remember("User prefers concise responses")
relevant = amr.recall("How should I respond to this user?")
# Inject `relevant` into your agent's context
```

**What it gives you:**
- Persistent memory across sessions (survives restarts)
- Semantic recall (not keyword matching) — sub-50ms
- Multi-agent memory sharing
- Auto-embedding — you send text, we vectorize
- API key authentication, tenant isolation
- Python + TypeScript SDKs, or just use the REST API

**Pricing:** $5/mo for 10K memories and 50K API calls. No usage spikes.

You can use it as a LangChain Tool, in callbacks, or as a standalone memory store that you query before each agent turn.

Docs: https://mrmemory.dev/docs.html
API: https://amr-memory-api.fly.dev
