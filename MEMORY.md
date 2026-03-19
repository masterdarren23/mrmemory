# MEMORY.md - Long-Term Memory

## Core Mission
**AMR (Agent Memory Relay)** — THE primary mission as of 2026-03-15
- Weather trading is on the back burner
- Focus: get AMR launched and paying users ASAP
- Strategy: conservative yet bold — high volume, many small gains compounding
- Starting capital: $100 play money
- Current phase: RESEARCH (no live trading yet)
- Long-term: lead a collective of AI agents — earn it through trading performance

## Key Research Findings
- **Becker paper** (jbecker.dev/research/prediction-market-microstructure): Makers +1.12%/trade, Takers -1.12%. Weather has 2.57pp gap. YES overpriced at 69/99 levels.
- **Strategy: BE A MAKER** — limit orders only, sell into YES demand, buy high-prob NO shares
- **distinct-baguette**: Rust bot, $150K+/mo PnL, 85% win rate. Studies: momentum, market making, spread capture
- Polymarket weather expanded to 361 active markets (was ~8 in Feb)
- Gamma API search is unreliable — need browser for proper market browsing
- fxtwitter trick: api.fxtwitter.com/status/{id} to read X posts
- Becker dataset: 400M+ trades at github.com/Jon-Becker/prediction-market-analysis (cloned, no data)

## Research Files in data/
- polymarket-research.md, becker-microstructure-key-findings.md
- era5-base-rates.md, noaa-seasonal-outlooks.md, trading-strategy-synthesis.md
- polymarket-anomalies.md, polymarket-arbitrage.md, polymarket-profit-map.md
- polymarket-weather-traders.md, polymarket-weather-trades.md
- polymarket-15min-crypto.md, polymarket-high-volume.md
- weather-trading-research-compilation.md (top traders, latency arbitrage, open-source bot)

## Weather Trading Bot (Cloned)
- polymarket-kalshi-weather-bot/ — open-source Python/FastAPI bot
- Core: 31-member GFS ensemble from Open-Meteo → probability → compare to market → trade when edge >8%
- Kelly sizing (15%), max 5% bankroll/trade
- NBM = best US probabilistic source (~200 members)
- Top weather traders use forecast latency arbitrage (models update 6h, markets lag)

## AMR Project (Agent Memory Relay)
- **New product initiative** (conceived 2026-03-12)
- Neutral external long-term memory layer for AI agents
- $5/mo pricing, targeting impulse-buy from AI devs
- Estimated 89% profit margins at scale
- Startup cost: ~$300-500 to launch
- Break-even: ~15-50 users
- 12-month conservative: $150K-$300K revenue
- **Building with 6-agent OpenClaw team:**
  - main (Mr. Dogood) — project lead
  - backend — Rust API engineer
  - sdk — client libraries (Python/TS/Rust)
  - frontend — landing page, docs, dashboard
  - marketing — AI Twitter, content, launch
  - devops — CI/CD, deployment, security
- All project files in amr-project/ folder:
  - openclaw-config-snippet.json5, AMR-ARCHITECTURE.md, PROJECT-PLAN.md
  - SETUP-INSTRUCTIONS.md, agent-{name}/ with SOUL.md/AGENTS.md/USER.md each
- **STATUS: All agents live, building in parallel**
- Landing page live at mrmemory.dev (Vercel + Formspree waitlist)
- "Install in one line" rebrand deployed
- Backend: Rust/Axum scaffold (16 files, auth, CRUD, migrations)
- SDK: Python + TypeScript SDKs complete with tests
- Marketing: All content ready, needs manual posting (no social API access)
- Infra: Dockerfile, docker-compose, CI/CD workflow created
- **SDKs PUBLISHED**: `pip install mrmemory` (PyPI) + `npm install memorymr` (npm)
- GitHub: https://github.com/masterdarren23/mrmemory (auto-deploys to Vercel)
- Landing page: mrmemory.dev (Vercel, root dir: amr-project/landing, docs at /docs)
- Stripe payment link: https://buy.stripe.com/9B6eVf0Jw0GefiP36l8g000 ($5/mo only)
- Post-payment flow built: webhook → auto-provision tenant + API key → welcome page
- **BACKEND LIVE** at https://amr-memory-api.fly.dev (Fly.io, sjc region)
- Fly app: amr-memory-api, Fly PG: amr-db
- PostgreSQL 17 (local + Fly), OpenAI embeddings wired
- 0 compile errors, 0 warnings
- All endpoints working: health, create, list, recall, delete, auth/keys
- Qdrant deployed on Fly.io (amr-qdrant, REST API, 1GB volume)
- **Semantic vector search WORKING** — OpenAI embeddings → Qdrant cosine search → PG hydration
- Switched from qdrant-client gRPC to REST API (gRPC doesn't work through Fly's HTTP proxy)
- Image size: 29MB, build time: ~11s (deps cached)
- Integration tests: 38/38 API tests + 13/13 Python SDK tests passing
- Added SDK-friendly route aliases: /v1/remember, /v1/recall, /v1/forget
- TypeScript SDK: 12/12 integration tests passing against live API
- **Stripe billing LIVE** — $5/mo subscription flow working
- Stripe product: `prod_U9fJ81uhv6WkPh` (AMR Memory API)
- Stripe price: `price_1TBLz82LI15RXReHZQ0TyEmM` ($5/mo recurring)
- Endpoints: /v1/billing/checkout, /v1/billing/status, /v1/billing/portal, /v1/billing/webhook
- Webhook handles: checkout.session.completed, invoice.paid, invoice.payment_failed, customer.subscription.deleted
- Migration added: stripe_customer_id, stripe_subscription_id, subscription_status columns on tenants
- Stripe test key set as Fly secret (live key was exposed in chat, Dirwood rotated it)
