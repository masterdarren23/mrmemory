# MrMemory — Cost & Revenue Breakdown
**Prepared by Darren W, Solo Founder | March 2026**

---

## Monthly Operating Costs

| Expense | Provider | Monthly Cost | Notes |
|---------|----------|-------------|-------|
| API Hosting (2 machines) | Fly.io | ~$5-10 | shared-cpu-1x, 256MB RAM each |
| PostgreSQL (managed) | Fly.io Postgres | ~$0 (dev) | Single node, included in free allowance |
| Qdrant (vector DB) | Fly.io | ~$3-5 | 1GB volume, single machine |
| Embeddings | OpenAI | ~$1-3 | text-embedding-3-small, $0.02/1M tokens |
| LLM calls | OpenAI | ~$1-5 | gpt-4o-mini for extraction/compression/judge |
| Domain (mrmemory.dev) | Registrar | ~$1 | ~$12/year |
| Landing page hosting | Vercel | $0 | Free tier (static HTML) |
| SSL/CDN | Fly.io + Vercel | $0 | Included |
| Monitoring | Fly.io dashboard | $0 | Built-in metrics |
| Analytics | Vercel Analytics | $0 | Free tier |
| **Total** | | **~$10-24/mo** | |

---

## Revenue

| Source | Status | Price | Notes |
|--------|--------|-------|-------|
| MrMemory API subscription | Live | $5/mo per user | 7-day free trial, Stripe billing |
| **Current MRR** | **$0** | | Pre-revenue, product just launched |

---

## Marketing Spend

| Channel | Spend | Results |
|---------|-------|---------|
| Reddit (organic posts) | $0 | 795 views, 146 unique cloners in 6 hours |
| GitHub (organic) | $0 | Public repo, README-driven discovery |
| Paid ads | $0 | None run |
| Influencer/affiliate | $0 | None |
| **Total marketing spend** | **$0** | |

---

## Unit Economics (Projected)

| Metric | Value |
|--------|-------|
| Price per user | $5/mo |
| Estimated cost per user | ~$0.50-0.80/mo (compute + embeddings + LLM) |
| Gross margin | ~84-90% |
| Break-even users | ~3-5 users (covers base infra) |
| Profitable at | ~15-50 users (covers dev time opportunity cost) |

**Cost scales with usage, not users.** Base infrastructure handles hundreds of users before needing to scale. Qdrant and Postgres grow linearly with stored memories. LLM costs are per-operation (auto_remember, compress, judge).

---

## Development Costs

| Item | Cost | Notes |
|------|------|-------|
| Developer time | $0 cash | Solo founder + AI agent team |
| OpenClaw (AI dev tool) | ~$50-100/mo | Anthropic API credits for development |
| Total development | ~$200-500 | Entire product built in ~3 weeks (March 2026) |

---

## Infrastructure Cost at Scale

| Users | Memories Stored | Monthly Infra | Monthly Revenue | Net |
|-------|----------------|---------------|-----------------|-----|
| 10 | ~50K | ~$25 | $50 | +$25 |
| 50 | ~250K | ~$50 | $250 | +$200 |
| 100 | ~500K | ~$80 | $500 | +$420 |
| 500 | ~2.5M | ~$200 | $2,500 | +$2,300 |
| 1,000 | ~5M | ~$400 | $5,000 | +$4,600 |

*Estimates assume average 5K memories/user, Fly.io scaling, OpenAI embedding costs.*

---

## Assets Included

- Complete Rust backend source code (production-deployed)
- Python SDK (published on PyPI as `mrmemory`)
- TypeScript SDK (published on npm as `memorymr`)
- Landing page + documentation site
- Stripe billing integration (subscriptions, webhooks, free trial)
- Docker Compose self-hosting setup
- Fly.io deployment configuration
- GitHub repository with full commit history
- Pitch deck
- All domain rights (mrmemory.dev)

---

## Key Financial Notes

1. **Pre-revenue** — Product launched March 2026, focus has been on building and organic distribution
2. **No debt or obligations** — No investors, no loans, no commitments
3. **Minimal burn** — ~$15/mo keeps everything running
4. **High margin product** — SaaS with 85%+ gross margins at any scale
5. **No tax filings yet** — No revenue generated to date
