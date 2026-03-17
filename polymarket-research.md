# Polymarket Weather Prediction Markets — Research Report

*Generated: February 15, 2026*

---

## Table of Contents
1. [How Polymarket Works](#1-how-polymarket-works)
2. [Current Weather & Climate Markets](#2-current-weather--climate-markets)
3. [Weather Data Sources for Edge](#3-weather-data-sources-for-edge)
4. [Common Mispricings in Weather Markets](#4-common-mispricings-in-weather-markets)
5. [Trading Strategies](#5-trading-strategies)

---

## 1. How Polymarket Works

### Core Mechanics
- **Binary outcome shares**: Every market has YES and NO shares, each priced $0.00–$1.00 USDC
- **Full collateralization**: Every YES + NO pair is backed by exactly $1.00 USDC
- **Share creation**: When a buyer offers $0.60 for YES and someone offers $0.40 for NO, $1.00 is split into 1 YES share + 1 NO share, distributed to respective buyers
- **Resolution payout**: Winning shares pay $1.00 each; losing shares become worthless
- **Peer-to-peer**: No "house" — you trade against other users. No banning for winning
- **Sell anytime**: Shares are liquid; you can exit before resolution at market price

### Price Discovery
- Prices = probabilities. A YES share at $0.35 = 35% implied probability
- **Displayed price** = midpoint of bid-ask spread (if spread ≤ $0.10), otherwise last traded price
- Prices are purely supply/demand driven — Polymarket doesn't set odds
- CLOB (Central Limit Order Book) model, not AMM

### Order Types
- **Limit orders**: Set your price, wait for a match. You're a "maker"
- **Market orders**: Fill at best available price. You're a "taker"

### Fees
- **Most markets: ZERO trading fees** — no deposit, withdrawal, or trading fees
- **Fee markets** (15-min crypto, NCAAB, Serie A only): Small taker fee, max ~1.56% at 50% probability, decreasing toward extremes
  - At $0.10 or $0.90: ~0.20%
  - At $0.05 or $0.95: ~0.06%
  - Fees fund the Maker Rebates Program (20-25% of fees redistributed to liquidity providers)
- **Weather markets currently have NO trading fees**

### Resolution
- Markets resolve per pre-defined rules (visible under each market's order book)
- Resolution requires a **proposal** backed by a USDC bond
- 2-hour challenge period after proposal
- Disputes handled by **UMA's optimistic oracle** — decentralized dispute resolution
- Resolution sources vary: official government data, specific APIs, news organizations

### Rewards Programs
- **Liquidity Rewards**: Earn daily USDC for placing competitive limit orders near the midpoint (within the "max spread"). Closer to midpoint = more rewards
- **Maker Rebates**: On fee-enabled markets, makers receive daily rebates from collected taker fees
- Minimum payout: $1/day

### API Access
- Full open-source API for market discovery, trading, and resolution
- GitHub: github.com/polymarket
- Developer docs: docs.polymarket.com/quickstart/overview
- Enables programmatic/algorithmic trading

### Platform Structure
- **International platform**: Not CFTC-regulated, operates globally
- **Polymarket US** (polymarket.us): Operated by QCX LLC, CFTC-regulated Designated Contract Market
- Settlement in USDC (USD stablecoin) on Polygon blockchain

---

## 2. Current Weather & Climate Markets

### Active Markets Found (as of Feb 15, 2026)

#### Hurricane Markets
| Market | Volume | Liquidity | Price | Ends |
|--------|--------|-----------|-------|------|
| Named storm forms before hurricane season? | $261K | $5.0K | 32% YES | ~May 2026 |
| Will a hurricane make landfall in the US by May 31? | $3.8K | $7.2K | 6% YES | ~May 2026 |
| Will a hurricane form by May 31? | $27.5K | $7.6K | 5% YES | ~May 2026 |
| Cat 4 hurricane US landfall before 2027? | $232K | $4.4K | 34% YES | ~Jan 2027 |
| Cat 5 hurricane US landfall before 2027? | $60.7K | $6.4K | 14% YES | ~Jan 2027 |

#### Temperature / Climate Markets
| Market | Volume | Liquidity | Price | Ends |
|--------|--------|-----------|-------|------|
| 2026 Feb: 1st, 2nd, 3rd hottest on record? | $228K | $12.8K | 80% "4th or lower" | ~Mar 2026 |
| Where will 2026 rank among hottest years? | $1M | $71.7K | 41% "2nd" | ~Jan 2027 |
| Will any month of 2026 be hottest on record? | $21.9K | $3.2K | 61% YES | ~Jan 2027 |

#### Natural Disaster
| Market | Volume | Liquidity | Price | Ends |
|--------|--------|-----------|-------|------|
| Natural Disaster in 2026? | $121K | $19.2K | 47% YES | ~Jan 2027 |

### Key Observations
- **Weather markets are NICHE** — volumes are $1K–$1M vs. political markets at $100M–$600M+
- **Liquidity is thin** — most weather markets have $3K–$20K liquidity (vs. millions in political)
- **No short-term weather markets** — no "Will it rain in NYC tomorrow?" type markets. Mostly seasonal/annual
- **Hurricane season** is the biggest weather category
- **No tornado, rainfall, flood, or snowfall markets** found
- Polymarket's weather coverage is sparse compared to its political/sports depth

---

## 3. Weather Data Sources for Edge

### Tier 1: Numerical Weather Prediction (NWP) Models

| Model | Provider | Resolution | Strengths | Access |
|-------|----------|------------|-----------|--------|
| **ECMWF IFS (HRES)** | European Centre | 9 km global | Gold standard for medium-range (3-15 day). Best overall global model | Paid API, some free via Open Data |
| **ECMWF ENS** | European Centre | 18 km, 51 members | Best ensemble system. Probabilistic forecasts with uncertainty | Same as above |
| **GFS** | NOAA/NWS (US) | 13 km global | Free, updated 4x daily, good for synoptic-scale | Free: nomads.ncep.noaa.gov |
| **GEFS** | NOAA | 25 km, 31 members | Free ensemble. Good for probability distributions | Free |
| **NAM** | NOAA | 3 km regional | Best for US short-range (1-3 day) detail | Free |
| **HRRR** | NOAA | 3 km, hourly | Best US short-range model. Updated hourly, 18-hr forecasts | Free |
| **Canadian GEM** | ECCC | 15 km global | Independent model, good for North Atlantic hurricanes | Free |

### Tier 2: AI/ML Weather Models (New Edge)

| Model | Provider | Notes |
|-------|----------|-------|
| **GraphCast** | Google DeepMind | Matches/beats ECMWF on many metrics. 0.25° resolution, 10-day forecasts |
| **Pangu-Weather** | Huawei | Competitive with ECMWF IFS at fraction of compute cost |
| **FourCastNet** | NVIDIA | Fast inference, good for ensemble generation |
| **GenCast** | Google DeepMind | Probabilistic, outperforms ENS on many metrics |
| **Aurora** | Microsoft | Foundation model, strong on diverse weather tasks |

### Tier 3: Specialized Sources

| Source | Use Case |
|--------|----------|
| **NHC (National Hurricane Center)** | Official US hurricane forecasts, track/intensity. Resolution source for hurricane markets |
| **NOAA CPC** | Seasonal outlooks (temperature, precipitation, drought). Key for annual climate markets |
| **NOAA NCEI** | Historical climate records. Resolution source for "hottest year" markets |
| **ERA5 Reanalysis** | ECMWF historical climate data. Base rates for any climate question |
| **Colorado State University** | Seasonal hurricane forecasts (April, June, August updates) |
| **ENSO/ONI data** | El Niño/La Niña indices. Major driver of seasonal climate patterns |
| **Climate Prediction Center** | Monthly/seasonal outlooks for temperature anomalies |

### How to Build an Edge

1. **Ensemble multi-model**: Don't rely on one model. Weight ECMWF, GFS, and AI models together
2. **Track model biases**: Each model has known systematic biases (e.g., GFS runs too warm in summer, ECMWF intensity bias in hurricanes)
3. **Historical base rates**: For seasonal markets, compute base rates from ERA5/NCEI data (e.g., "How often does a Cat 5 hit the US in any given year?" → ~5-10%)
4. **ENSO state matters enormously**: El Niño suppresses Atlantic hurricanes; La Niña enhances them. Current ENSO state shifts probabilities by 20-50%
5. **Lead time advantage**: Markets are slow to update. If ECMWF 00Z run shows a major shift, the market may not reflect it for hours

---

## 4. Common Mispricings in Weather Markets

### Why Weather Markets Misprice

1. **Low liquidity = slow price discovery**: With $3K–$20K in liquidity, even small informed bets move the market. But getting in/out is hard — wide spreads
2. **Base rate neglect**: Participants anchor on recent events. After an active hurricane season, the next year's Cat 5 probability gets bid up beyond base rates
3. **Recency bias**: "It was a warm January, so 2026 will be the hottest year" — ignoring mean reversion and ENSO transitions
4. **Narrative-driven pricing**: Media coverage of climate change → people overbid "hottest year" markets. The signal is real but the magnitude often gets priced in early
5. **Calendar effects**: Hurricane markets get attention in peak season (Aug-Oct) but are neglected in off-season, where informed traders can build positions cheaply

### Specific Mispricing Patterns

#### Hurricane Markets
- **Pre-season overpricing**: Named storms "before hurricane season" (currently 32%) is likely overstated — pre-season named storms happen ~20-25% of years historically
- **Cat 5 landfall**: 14% for Cat 5 US landfall in 2026 — historical base rate is ~3-5% per year. Likely overpriced due to availability bias
- **Cat 4 landfall**: 34% for Cat 4 US landfall — historical base rate is ~10-15% per year. Also likely overpriced
- **Key driver**: Check the current ENSO state. If neutral/El Niño developing, Atlantic activity suppressed → these are even more overpriced

#### Temperature/Climate Markets
- **"Hottest year" ranking**: Highly path-dependent on ENSO. After a strong El Niño year (2024-2025), reversion to La Niña/neutral typically means cooler anomalies
- **Monthly records**: 61% for "any month of 2026 hottest on record" — plausible given long-term warming trend, but depends on whether 2024-2025 El Niño spike has passed
- **February record**: 80% that Feb 2026 is NOT top 3 hottest — this is near-term and NWP models give strong signal. Check if current global temperature anomalies support this

#### General Patterns
- Markets with **multi-outcome brackets** (e.g., "Where will 2026 rank?") often have individual outcomes that don't sum to 100% correctly — arbitrage opportunity
- **Time decay**: Long-dated markets (ending Dec 2026) have capital lockup costs. A 34% YES that's truly 15% still takes 10 months to resolve — opportunity cost matters

---

## 5. Trading Strategies

### Strategy 1: Informed Market Making (Best for Weather)

**Concept**: Provide liquidity on weather markets using superior forecast data, earning both the spread and liquidity rewards.

- Place limit orders on both YES and NO sides around your model-derived fair value
- Earn Polymarket's **Liquidity Rewards** (daily USDC payouts for competitive orders within max spread)
- In thin weather markets, you may be one of few market makers → high reward share
- **Risk**: Your model could be wrong; manage position limits
- **Edge**: Weather markets have so few sophisticated participants that even basic model consensus gives you an edge over the marginal trader

**Example**: If your models say Cat 5 US landfall probability is 5%, but market shows 14%:
- Place NO limit orders at $0.85 (= 15% implied YES)
- If filled, you have 85¢ shares that pay $1.00 at ~95% probability → ~18% expected return
- Also collect liquidity rewards while your orders sit in the book

### Strategy 2: Event-Driven Trading

**Concept**: React quickly to forecast updates that shift probabilities.

- Monitor NWP model runs (GFS: 00Z/06Z/12Z/18Z; ECMWF: 00Z/12Z)
- When a model run shows a significant change (e.g., tropical development signal appears), trade before the market catches up
- **Key timing**: ECMWF 00Z data available ~06:00 UTC. Markets are thin overnight → your order may sit for hours but will fill when Europeans/Americans wake up
- Use the Polymarket API for programmatic order placement

**Example workflow**:
1. ECMWF 00Z shows tropical invest in Caribbean gaining organization
2. GFS confirms at 06Z
3. Buy YES on "hurricane forms by May 31" at 5¢ before social media/news picks it up
4. NHC issues Tropical Weather Outlook 2 days later → market jumps to 25¢
5. Sell for 5x return

### Strategy 3: Base Rate Arbitrage

**Concept**: Identify markets where crowd pricing deviates significantly from climatological base rates.

- Build a database of historical base rates for every type of weather market question
- Compare to current Polymarket prices
- Trade the deviation when it exceeds your edge threshold (typically >10 percentage points after accounting for liquidity costs)
- **Best for**: Long-dated seasonal markets in the off-season when attention is low

### Strategy 4: Cross-Outcome Arbitrage

**Concept**: Exploit pricing inconsistencies in multi-outcome markets.

- In "Where will 2026 rank among hottest years?" — sum all outcome probabilities
- If they sum to >100% (common in illiquid markets), sell overpriced outcomes
- If <100%, buy underpriced outcomes
- **Also**: Cross-market arbitrage — if "2026 hottest year" and "2026 top 3 hottest" are both live, ensure prices are internally consistent

### Strategy 5: Seasonal Calendar Trading

**Concept**: Exploit predictable attention cycles in weather markets.

- **Hurricane markets**: Cheap in winter (Jan-Apr), expensive in summer (Jul-Oct)
- Buy during the inattention period when prices drift toward extremes
- Sell during peak attention when media coverage brings in retail traders who overpay
- **Temperature markets**: Similar — annual record markets get attention in summer/fall

### High-Frequency Considerations

True HFT isn't viable on Polymarket weather markets due to:
- **Thin liquidity**: $3K–$20K means even $500 trades move price
- **Slow resolution**: Most weather markets resolve over months, not minutes
- **Blockchain settlement**: Orders go through Polygon — not microsecond latency

**However**, "high-frequency" relative to weather forecasting means:
- Updating positions 4x/day on NWP model cycles (every 6 hours)
- Reacting within minutes to NHC advisories, CPC updates, and model data releases
- Programmatic order management via the API to maintain tight quotes

### Risk Management

- **Position sizing**: Never more than 5-10% of capital in a single weather market (thin liquidity makes exit hard)
- **Correlation**: Hurricane markets are correlated with each other. Don't treat Cat 4 and Cat 5 landfall as independent bets
- **Capital lockup**: Long-dated markets tie up capital. Factor in opportunity cost (~5% annual risk-free rate)
- **Resolution risk**: UMA oracle disputes are rare but possible. Understand the resolution criteria precisely
- **Model risk**: No model is perfect. Size bets proportional to your confidence in the model-market divergence

---

## 6. Key Takeaways & Recommendations

1. **Weather is a niche opportunity on Polymarket** — low competition but also low liquidity. Good for small-to-medium sized informed traders, not for large capital deployment

2. **The biggest current edge** is in hurricane markets, which appear systematically overpriced relative to climatological base rates (Cat 4 at 34% vs ~12% base, Cat 5 at 14% vs ~4% base)

3. **Best data stack**: ECMWF (paid) + GFS (free) + one AI model (GraphCast/GenCast) + NOAA CPC seasonal outlooks + ERA5 historical base rates

4. **Market making + liquidity rewards** is likely the most consistent income strategy given the thin liquidity

5. **Programmatic trading via API is essential** — weather data updates on fixed schedules (model runs every 6h), making automated monitoring and order placement highly valuable

6. **Watch for Polymarket expanding weather coverage** — if they add short-term markets (daily temperature, weekly precipitation), the opportunity set grows dramatically

7. **Start by paper trading** — track your model-derived probabilities against market prices for 2-3 months to calibrate your edge before deploying real capital

---

## Appendix: Useful Links

- Polymarket: https://polymarket.com
- Polymarket API Docs: https://docs.polymarket.com/quickstart/overview
- Polymarket GitHub: https://github.com/polymarket
- GFS Data: https://nomads.ncep.noaa.gov
- ECMWF Open Data: https://www.ecmwf.int/en/forecasts/datasets/open-data
- NOAA CPC Outlooks: https://www.cpc.ncep.noaa.gov
- NHC: https://www.nhc.noaa.gov
- NOAA NCEI (climate records): https://www.ncei.noaa.gov
- CSU Tropical Forecast: https://tropical.colostate.edu
- GraphCast: https://deepmind.google/discover/blog/graphcast-ai-model-for-faster-and-more-accurate-global-weather-forecasting/
