# memorymr - Agent Memory Relay

[![npm](https://img.shields.io/npm/v/memorymr)](https://www.npmjs.com/package/memorymr)
[![License](https://img.shields.io/badge/license-MIT-blue)](LICENSE)
[![Docs](https://img.shields.io/badge/docs-mrmemory.dev-8b7aff)](https://mrmemory.dev/docs.html)

Persistent long-term memory for AI agents. One line to install, three lines to integrate.

**[Docs](https://mrmemory.dev/docs.html)** · **[API Reference](https://mrmemory.dev/docs.html#endpoints)** · **[Website](https://mrmemory.dev)**

## Install

```bash
npm install memorymr
```

## Quickstart

```typescript
import { AMR } from 'memorymr'

const amr = new AMR('amr_sk_...')  // or set AMR_API_KEY env var

// Store a memory
await amr.remember('User prefers dark mode and vim keybindings')

// Semantic recall
const memories = await amr.recall('What are the user preferences?')
for (const m of memories) {
  console.log(m.content, m.score)
}

// Forget a memory
await amr.forget(memories[0].id)
```

## Configuration

```typescript
const amr = new AMR({
  apiKey: 'amr_sk_...',       // or set AMR_API_KEY env var
  agentId: 'my-assistant',
  namespace: 'default',
  timeout: 10_000,             // ms
  maxRetries: 3,
})
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

Starts at **$5/mo** - 10K memories, 50K API calls. [Sign up →](https://amr-memory-api.fly.dev/v1/billing/checkout)

## Links

- **Docs:** https://mrmemory.dev/docs.html
- **Dashboard:** https://mrmemory.dev
- **GitHub:** https://github.com/amr-dev/amr-typescript

## License

MIT
