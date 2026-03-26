# memorymr - Agent Memory Relay

[![npm](https://img.shields.io/npm/v/memorymr)](https://www.npmjs.com/package/memorymr)
[![License](https://img.shields.io/badge/license-MIT-blue)](LICENSE)
[![Docs](https://img.shields.io/badge/docs-mrmemory.dev-8b7aff)](https://mrmemory.dev/docs)

Persistent long-term memory for AI agents. One line to install, three lines to integrate.

**[Docs](https://mrmemory.dev/docs)** · **[API Reference](https://mrmemory.dev/docs#endpoints)** · **[Website](https://mrmemory.dev)**

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

## Auto-Remember

Extract memories from conversations automatically:

```typescript
const result = await amr.autoRemember([
  { role: 'user', content: 'I love hiking and my favorite language is Rust' },
  { role: 'assistant', content: 'Great choices!' },
], { sync: true })

console.log(result) // { extracted: 2, created: 2, ... }
```

## Compress

Compress related memories into denser representations:

```typescript
const result = await amr.compress({ namespace: 'default', sync: true, dryRun: true })
console.log(`Would compress ${result.groups_compressed} groups`)
```

## Self-Edit Tools

Update, bulk-delete, and merge memories:

```typescript
// Update an existing memory (re-embeds automatically)
await amr.update('mem_abc123', { content: 'Updated preference', tags: ['preference'] })

// Bulk delete old memories
const result = await amr.deleteOutdated({ olderThanSeconds: 86400 * 30, dryRun: true })
console.log(`Would delete ${result.deleted} memories`)

await amr.deleteOutdated({ olderThanSeconds: 86400 * 30, tags: ['ephemeral'] })

// Merge memories (LLM summarization)
const merged = await amr.merge(['mem_abc123', 'mem_def456'])
console.log(merged.content) // Summarized

// Or provide your own content
const merged2 = await amr.merge(['mem_abc123', 'mem_def456'], { content: 'Custom merged content' })
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
| POST | `/v1/memories/auto` | LLM auto-remember |
| POST | `/v1/memories/compress` | Compress related memories |
| PATCH | `/v1/memories/:id` | Update a memory |
| DELETE | `/v1/memories/outdated` | Bulk delete by age/tags |
| POST | `/v1/memories/merge` | Merge memories into one |

Auth: `Authorization: Bearer amr_sk_...`

## Pricing

Starts at **$5/mo** - 10K memories, 50K API calls. [Sign up →](https://amr-memory-api.fly.dev/v1/billing/checkout)

## Links

- **Docs:** https://mrmemory.dev/docs.html
- **Dashboard:** https://mrmemory.dev
- **GitHub:** https://github.com/masterdarren23/mrmemory

## License

MIT
