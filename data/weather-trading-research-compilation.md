# Polymarket Weather Trading Research Compilation

**Compiled:** 2026-03-12 | **Purpose:** Actionable intelligence for weather prediction market trading

---

## Table of Contents
1. [Market Structure & Mechanics](#1-market-structure--mechanics)
2. [Successful Traders & Profiles](#2-successful-traders--profiles)
3. [Core Trading Strategies](#3-core-trading-strategies)
4. [Weather Data Edge & Sources](#4-weather-data-edge--sources)
5. [Algorithmic & Bot Trading](#5-algorithmic--bot-trading)
6. [Market Inefficiencies & Biases](#6-market-inefficiencies--biases)
7. [Distinct-Baguette Deep Dive](#7-distinct-baguette-deep-dive)
8. [Tools, Platforms & Resources](#8-tools-platforms--resources)
9. [Kalshi-Specific Intelligence](#9-kalshi-specific-intelligence)
10. [Global Temperature Anomaly Markets](#10-global-temperature-anomaly-markets)

---

## 1. Market Structure & Mechanics

### Polymarket Weather Markets
- **198 active daily temperature markets** and **372 total weather markets** as of March 2026
- Cities covered: NYC, London, Seoul, Buenos Aires, Shanghai, Hong Kong, and expanding
- Markets structured as **multi-outcome buckets** (e.g., temperature ranges like 4-5°C, 6-7°C)
- Share prices range **$0.01 to $0.99**, resolving at $1.00 for correct outcome, $0.00 for incorrect
- **No special taker fees** on weather markets (unlike 15-min crypto markets)
- Resolution sources are **fixed and publicly known** in advance (e.g., Weather Underground EGLC station for London)

### Kalshi Weather Markets
- 4 primary cities: NYC (KNYC/Central Park), Chicago (KMDW/Midway), Miami (KMIA), Austin (KAUS)
- Also: Denver (KDEN), Houston (KHOU), Philadelphia (KPHL), Los Angeles (KLAX)
- **6 brackets per market**: middle 4 are 2°F wide, edge brackets catch everything above/below
- Markets launch at **10 AM the previous day**
- Resolution via **NWS Daily Climatological Report (CLI)**
- CLI reporting period: **12:00 AM to 11:59 PM Local Standard Time** (NOT daylight saving time)
- Settlement uses final official CLI report, not mid-day preliminary reports

### Key Resolution Details (Critical for Trading)
- **NWS stations have TWO types**: hourly stations and 5-minute stations
- **Hourly stations**: Record constantly, report once per hour, temp recorded to nearest 0.1°F → converted to 0.1°C → converted back to °F (introduces minor rounding error)
- **5-minute stations**: Record 1-minute averages → average 5 of those → round to nearest whole °F → convert to nearest whole °C (due to DOS limitations!). This introduces **significant rounding error of 1°F or more**
- **The daily high** is determined by checking BOTH station types and taking the higher of the two
- The highest value is rounded to nearest whole °F **without** the C→F conversion error
- **Implication**: The real-time 5-minute data you see on NWS can be off by 1°F+ from the official daily high. The daily high can be HIGHER than any value displayed in the 5-minute time series

---

## 2. Successful Traders & Profiles

### gopfan2 — $2M+ Profit
- Most profit from weather markets specifically
- Strategy: Buy YES shares when priced **below $0.15**, buy NO shares when priced **above $0.45**
- Limits risk to **~$1 per position**
- Makes thousands of small bets with high accuracy rate
- Profits compound through volume, not individual bet size

### Hans323 — $1.1M Single Trade
- Turned a few hundred dollars into tens of thousands
- Famous trade: **$92,000 bet on London weather** at ~8% implied probability → paid $1.11M
- Core edge: **Forecast latency** — faster than other participants at incorporating new weather model data
- "Bets heavily on low-probability events with asymmetric risk-reward ratios"

### securebet — $7 → $640 (9,000%+ ROI)
- Temperature markets in NYC and Seattle
- 3,000+ individual predictions using tiny bet sizes
- Consistently referenced NOAA weather data

### meropi — $30K Profit (Automated)
- Fully automated $1-$3 micro bets
- Sometimes enters positions at **$0.01 per share** (500x potential payoff)
- Bot monitors latest model outputs vs. Polymarket prices in real time

### 1pixel — $18.5K from $2.3K Deposits
- Trades only NYC and London weather markets
- Individual trades: $6 → $590, $15 → $547
- Catches mispriced ranges

### neobrother — $20K+
- Focuses on Buenos Aires and NYC daily high temperatures
- **"Temperature laddering" strategy**: places low-cost bets across a range of temperatures
- High returns when actual temperature falls within the range

---

## 3. Core Trading Strategies

### Strategy 1: Forecast Latency Arbitrage ⭐
**The #1 edge exploited by top traders**

- Weather models (GFS, ECMWF) update on **fixed 6-hour schedules** (GFS: 00, 06, 12, 18 UTC; ECMWF: 00, 12 UTC)
- When new model run shows significant forecast shift (2°F+ change), Polymarket prices **lag behind**
- Window of opportunity: minutes to hours before other traders notice
- Buy shares in the range the updated forecast favors at stale (too-cheap) prices
- Can **exit before resolution** once market reprices, or hold to settlement
- This is pure **latency arbitrage**: fast data (professional forecasts) vs. slow data (human crowd)

### Strategy 2: Model Consensus Trading
- Compare **3+ independent models**: GFS, ECMWF, ICON, CMC, GEM
- When all models agree on a temperature range, probability is typically **70-90%**
- Only act when model consensus is strong AND market price implies much lower probability
- Example: 3 models show 6-7°C for London, market prices that range at $0.15 → massive edge
- **Rule of thumb**: Only trade when 3+ models agree and market is significantly underpriced

### Strategy 3: Statistical Micro-Betting
- Hundreds/thousands of $1-$3 positions
- Survives variance — no single loss is meaningful
- Builds feedback loop for learning which setups work
- **gopfan2 rules**: Buy YES < $0.15 with strong model support; Buy NO > $0.45 when models disagree
- Win rates: 50-80% with small losses, large winners

### Strategy 4: Temperature Laddering (neobrother)
- Place small bets across multiple adjacent temperature ranges
- Ensures you capture the actual outcome within your spread
- Works especially well when forecast uncertainty is moderate

### Strategy 5: Low-Probability Tail Bets (Hans323)
- Identify situations where unlikely outcomes are mispriced
- Extreme weather events (heat waves, cold snaps) often underpriced
- Asymmetric payoff: risk small amounts for potential 10-100x returns
- Requires understanding of model uncertainty and tail risks

### Strategy 6: Spread Capture / Market Making
- Buy BOTH Yes and No when combined price < $1.00
- Lock in risk-free profit regardless of outcome
- Works best in low-liquidity or newly launched markets
- distinct-baguette's approach in crypto markets (see Section 7)

---

## 4. Weather Data Edge & Sources

### Free Forecast Data Sources
| Source | What It Provides | URL |
|--------|-----------------|-----|
| **Tropical Tidbits** | Raw GFS, ECMWF model output in visual format | tropicaltidbits.com |
| **Windy.com** | "Compare models" feature — GFS, ECMWF, ICON side-by-side | windy.com |
| **Ventusky** | All weather models in one module | ventusky.com |
| **NOAA Climate Data Online** | Historical weather data for baseline understanding | climate.data.noaa.gov |
| **Open-Meteo** | 31-member GFS ensemble forecasts (free API, no auth) | open-meteo.com |
| **NWS Time Series** | Near real-time station observations | weather.gov |
| **NWS CLI Reports** | Official resolution data for Kalshi | weather.gov |

### Model Update Schedule (Critical Timing)
| Model | Update Times (UTC) | Notes |
|-------|-------------------|-------|
| **GFS** | 00, 06, 12, 18 | Most accessible, 31-member ensemble available |
| **ECMWF** | 00, 12 | Generally considered most accurate |
| **ICON** | 00, 06, 12, 18 | German model, good for European cities |
| **CMC/GEM** | 00, 12 | Canadian model |
| **ECMWF-AI** | Varies | AI-enhanced ECMWF variant |

### Key Data Principles
1. **Always check the specific resolution station** — don't use Heathrow data if market resolves on EGLC (London City Airport)
2. **Phone weather apps are NOT sufficient** — use professional model output
3. **1-2 day forecasts are 85-90% accurate** — this is the sweet spot for trading
4. **Historical data** helps you identify unrealistic market prices (e.g., 60°F in NYC in January)
5. **Ensemble spread** (how much the 31 GFS members disagree) indicates forecast confidence

### Advanced: NWS Station Data Interpretation
- Real-time NWS data has known **rounding/conversion artifacts**
- 5-minute station data can be off by 1°F+ from official daily high
- The official high uses raw readings WITHOUT the C↔F conversion error
- Understanding this gives an edge when trading based on real-time observations
- **Climate Sight** is researching ML models trained on co-located sensors near ASOS stations to correct for these errors

---

## 5. Algorithmic & Bot Trading

### Open-Source Bot: polymarket-kalshi-weather-bot (GitHub)
- **Author**: suislanchez | **Reported profits**: $1,325 as of March 9, 2026
- **URL**: github.com/suislanchez/polymarket-kalshi-weather-bot
- Python + React dashboard, MIT licensed, 100% free to run
- **Weather Strategy**:
  - Fetches 31-member GFS ensemble from Open-Meteo (free, no auth)
  - Counts fraction of ensemble members above/below market threshold
  - That fraction = model probability
  - Trades when **edge > 8%**
  - Scans every 5 minutes
- **Position Sizing**: Fractional Kelly (15%) with caps at $100/weather trade
- **Risk Management**: Daily $300 loss limit, max 20 open positions
- Also trades BTC 5-min markets (edge > 2%)
- Cities: NYC (KNYC), Chicago (KORD), Miami (KMIA), LA (KLAX), Denver (KDEN)

### Bot Architecture Pattern (Common Approach)
1. **Every 2-5 minutes**: Fetch latest forecast model data
2. **Compare**: Model probability vs. market implied probability
3. **If edge > threshold**: Place limit order at market price + $0.01
4. **Position sizing**: Kelly criterion or fixed small amounts ($1-$3)
5. **Safety**: Daily loss limits, max positions, dry-run mode

### Polyforecast.io — Signal Service
- Run by James Cole, former climate data analyst (Denver, CO)
- Runs **5 independent forecast models** (GFS, ECMWF, ECMWF-AI, ICON, GEM) against every Polymarket weather market
- Trades his own money on every signal
- Free daily email with: city, temp range, market price vs ensemble probability, trade/pass decision
- Full transparency: all positions verifiable on-chain (Polygon)
- Tracks ROI, win rate, profit factor, complete trade history

### Key Bot Insight
> "The bot uses a better source of information (NOAA forecasts from actual meteorological science) to find mispriced shares on Polymarket (set by regular people who are often just guessing). And because the bot runs automatically, it catches these opportunities 24 hours a day."

### Why Bots Still Have Edge
- Most bettors are **casual** — guessing based on weather apps or gut feeling
- Weather markets are **relatively new and growing** — new casual bettors enter daily
- Bots run 24/7 and catch latency windows humans miss
- As more bots enter, margins will shrink — **current window is optimal**

---

## 6. Market Inefficiencies & Biases

### Documented Inefficiencies

1. **Forecast Latency Gap**: Market prices lag behind professional model updates by minutes to hours
2. **Casual Bettor Bias**: Most participants use phone weather apps, not professional forecast models
3. **Favourite-Longshot Bias**: Well-documented in prediction markets — low-probability outcomes tend to be overpriced, high-probability outcomes underpriced (per Lionel Page & Robert Clemen research)
4. **New Market Inefficiency**: Newly launched weather markets (new cities) have wider mispricings
5. **Low Liquidity = Wider Spreads**: Combined Yes+No prices sometimes sum to < $1.00, enabling risk-free spread capture
6. **Resolution Source Confusion**: Many traders don't know the exact station used for resolution, leading to bets based on wrong location data
7. **Time Zone Confusion**: DST vs LST vs UTC confusion causes traders to misunderstand reporting periods

### Academic Support
- **Marginal Revolution** (Oct 2025): Polymarket has Brier scores of 0.0256 at 12 hours pre-resolution — "hard to overstate how impressive that is" — but found "small but systematic errors" with favourite-longshot bias
- **ScienceDirect** (2018): "Prediction market prices may not incorporate all information regarding environmental factors such as weather and atmospheric conditions" — suggesting weather-related inefficiencies exist
- Markets with >$1M volume have Brier scores of 0.0159 at one day prior

### Self-Correcting But Slow
- Any attempt to distort markets creates profit opportunities for informed traders
- Weather markets self-correct but **slowly enough** to trade the latency
- More sophisticated traders entering → margins compressing over time

---

## 7. Distinct-Baguette Deep Dive

### Profile Summary
- **Polymarket profile**: polymarket.com/@distinct-baguette
- **Wallet**: 0xe00740bce98a594e26861838885ab310ec3b548c
- **Joined**: October 2025
- **Stats**: 42,157 predictions, $11.4K positions value, $11.4K biggest win, 267.6K views
- **ScanWhale rating**: FISH tier (+$4.8K 30D PnL, **85% win rate**, CRYPTO specialist)
- **PolygonScan**: $406,761.51 in tokens (13 tokens)
- **Reported total profit**: $240,000-$448,000+ since October 2025

### Strategy (NOT Weather — Crypto Markets)
**distinct-baguette is a crypto UP/DOWN bot, not a weather trader.** Key distinction.

Per @thejayden's X thread (Dec 10, 2025, 27.5K views, 412 bookmarks):
> "Another @Polymarket trader making $170k/month. Today it's about distinct-baguette. A bot nothing like gabagool22. I've spent 2 days cracking his algo. $150k+/month PnL, 10,736 prediction markets, low volume, low trades, insane efficiency."

### Core Strategy: Spread Capture on Crypto 15-Minute Markets
Per @herman_m8 (Matys) on X:
- Focuses **exclusively on 15-minute crypto markets** (BTC, ETH, SOL, XRP)
- Places limit orders: **YES at $0.45-$0.47** and **NO at $0.46-$0.48**
- Once both sides fill, total cost stays **under $1.00**
- After settlement, one side pays $1.00
- **Direction doesn't matter** — edge comes from ~5% spread capture
- "No predictions, no gambling. Just basic arithmetic applied every 15 minutes."
- Earns **>$6,000/day**

### Three Strategies (from distinct-baguette.com)
1. **Momentum**: Follows directional BTC signals from Binance in sub-second (4 execution modes)
2. **Market Making**: Two-sided quoting with Binance preemptive cancel (detects adverse price moves, cancels before getting filled)
3. **Spread Capture**: Arbitrages the gap between UP and DOWN sides for risk-neutral profit

### Bot Technical Details
- Written in **Rust**
- Runs on EU VPS ($6-12/mo, DigitalOcean/Vultr Amsterdam)
- Trades across BTC, ETH, SOL, XRP in 5m/15m/1h windows
- Includes stop-loss, cooldown, partial exit, dry-run mode, graceful restarts
- Full Polymarket CLOB + vendored SDK
- **6.8 GB historical data** (11,201 market files with orderbook tick history at 100ms resolution)
- Linked to gabagool22 — "Probably the same person — same patterns, same timing"

### Selling the Bot
- distinct-baguette.com sells the bot for **$199** (one-time)
- Includes full Rust source, 6.8 GB data, 10 pre-tuned configs, deployment toolkit, dashboard
- Claims $500,000+ profit (verifiable via wallet)
- 4 verified buyers listed
- Proprietary license, one deployment per purchase
- Contact: distinct-baguette@protonmail.com

### Relevance to Weather Trading
While distinct-baguette trades crypto, the **spread capture** and **market making** principles apply to weather markets:
- Look for weather markets where Yes + No < $1.00
- Use limit orders to capture the spread
- The automation and speed principles are directly transferable

---

## 8. Tools, Platforms & Resources

### Weather Trading Platforms/Tools
| Tool | Description | URL |
|------|-------------|-----|
| **Wethr.net** | Weather market trading intelligence, multi-station monitoring, advanced model analysis | wethr.net |
| **Climate Sight / Weatherbetter** | ML-powered arbitrage signals, real-time data for Kalshi/Polymarket | climatesight.app / weatherbetter.app |
| **Polyforecast.io** | Free daily trading signals from 5-model ensemble | polyforecast.io |
| **ScanWhale** | Whale tracking and alpha signals on Polymarket | scanwhale.com |
| **Polymarket Analytics** | Real-time prediction market data and dashboards | polymarketanalytics.com |

### Useful Data Sources for Resolution
| City | Kalshi Station | Polymarket Resolution |
|------|---------------|----------------------|
| NYC | KNYC (Central Park) | Weather Underground / NWS |
| London | N/A (Kalshi) | Weather Underground EGLC (London City Airport) |
| Chicago | KMDW (Midway) | NWS CLI |
| Miami | KMIA (Miami Intl) | NWS CLI |
| Austin | KAUS (Bergstrom) | NWS CLI |
| Denver | KDEN (Denver Intl) | NWS CLI |
| Houston | KHOU (Hobby) | NWS CLI |
| Philadelphia | KPHL (Phila Intl) | NWS CLI |

### Subreddits & Communities
- r/Kalshi — Active temperature market discussion
- r/PolymarketTrading — Weather market threads
- r/PredictionsMarkets — General prediction market discussion
- r/polymarketAnalysis — Strategy and bot discussion

---

## 9. Kalshi-Specific Intelligence

### From Kalshi's Own Trading Guide (news.kalshi.com)
- Start with NWS hourly forecasts, Weather.com, Apple Weather
- **Next level**: Use weather models via Ventusky to find divergences from forecasts
- **Trend detection** is key — e.g., wildfire haze causing temps to undershoot forecasts
- "If there are a lot of cheap shares available in a bracket that I can legitimately justify as a possible winner, I'm buying them every time"
- "Traders tend to value certainty before they should" — exploit this by taking swings on underpriced brackets
- **Know your resolution sources**: Central Park, Midway Airport, Miami International, Austin-Bergstrom
- Know when the high appears first, how DST applies, and what the resolution source is

### From r/Kalshi Temperature Guide
- Meteorologist advice: "I average around 1-2°F off on high temperatures. It's probably easier to bet on the nil than try to nail the temperature."
- Focus on **high accuracy near-term forecasting** rather than real-time NWS time series trading
- The conversion errors in 5-minute station data make real-time trading treacherous without deep understanding

---

## 10. Global Temperature Anomaly Markets

### Monthly Global Temperature Anomaly (Advanced)
From PolyMaster on Medium (polymaster.io):
- Markets like: "Will the global temperature anomaly for September 2025 exceed 1.19°C?"
- Resolution based on NASA GISTEMP v4 data (released mid-month following)
- **GISTEMP v4 is completely open source** — all data public, all algorithms transparent
- Data pipeline: 25,000+ weather stations → GHCN v4 (quality controlled) → ERSST v5 (sea surface) → 6-step processing → global anomaly value
- **Key insight**: You can **reproduce NASA's calculations** using their open-source code before official release
- Data sources: GHCN v4 land station data + ERSST v5 sea surface temperature
- Processing: Quality control → Urban heat island adjustment → 1200km gridding → Regional averaging
- Tool available at polymaster.io for backtesting
- **Edge**: Calculate the anomaly yourself using publicly available data before NASA releases it

---

## Actionable Summary: How to Start Trading Weather Markets

### Immediate Steps
1. **Pick 1-2 cities** (NYC + London recommended for liquidity)
2. **Set up data sources**: Tropical Tidbits + Windy.com + NWS (all free)
3. **Learn model update times**: GFS at 00/06/12/18 UTC, ECMWF at 00/12 UTC
4. **Start with $100**, micro-bets of $1-3

### Daily Routine
1. Check model consensus (GFS, ECMWF, ICON) for tomorrow's high temp
2. Compare to Polymarket prices for each temperature bucket
3. If 3+ models agree but market price is below $0.15 for that range → buy YES
4. If clearly unrealistic range is priced above $0.45 → buy NO
5. After model updates, check for significant forecast shifts and trade the latency

### Scaling
1. Track every trade, build a win-rate database
2. After proving positive edge over 100+ trades, increase position sizes
3. Consider automation via the open-source bot (suislanchez GitHub) or Polyforecast signals
4. Explore spread capture opportunities (Yes+No < $1.00)

### Risk Management
- Never more than 5% of bankroll per trade
- Daily loss limit of 20%
- Max 5-20 simultaneous positions
- Half-Kelly sizing for safety
- Always start with dry-run/paper trading

---

## Source Links

1. [Medium: Making Millions Betting Weather](https://ezzekielnjuguna.medium.com/people-are-making-millions-on-polymarket-betting-on-the-weather-and-i-will-teach-you-how-24c9977b277c) — Ezekiel Njuguna, Jan 2026
2. [Dev Genius: Weather Trading Bots Making $24K](https://blog.devgenius.io/found-the-weather-trading-bots-quietly-making-24-000-on-polymarket-and-built-one-myself-for-free-120bd34d6f09) — Ezekiel Njuguna, Feb 2026
3. [Phemex: Polymarket Traders Profit from Weather](https://phemex.com/news/article/polymarket-traders-profit-from-weather-predictions-58213) — Feb 2026
4. [r/Kalshi: Incomplete Guide to Temperature Markets](https://www.reddit.com/r/Kalshi/comments/1hfvnmj/an_incomplete_and_unofficial_guide_to_temperature/) — Dec 2024
5. [Kalshi Blog: Trading the Weather](https://news.kalshi.com/p/trading-the-weather) — Aug 2025
6. [Wethr.net Trading Guide](https://wethr.net/edu/trading-guide)
7. [Polyforecast.io](https://polyforecast.io) — Free daily signals
8. [PolyMaster: Global Temperature Anomaly Edge](https://medium.com/@wanguolin/how-to-gain-an-edge-in-the-global-temperature-anomaly-market-1-from-weather-station-to-polymarket-c6e6fdc9444a) — Nov 2025
9. [GitHub: polymarket-kalshi-weather-bot](https://github.com/suislanchez/polymarket-kalshi-weather-bot) — Open source bot
10. [Climate Sight: Weather Prediction Market Strategies](https://www.climatesight.app/weather-prediction-market-strategies/)
11. [distinct-baguette.com](https://distinct-baguette.com) — Bot for sale
12. [X @thejayden on distinct-baguette](https://x.com/thejayden/status/1998688327327732132) — Dec 2025 analysis
13. [ScanWhale: distinct-baguette profile](https://www.scanwhale.com/traders/0xe00740bce98a594e26861838885ab310ec3b548c)
14. [Marginal Revolution: Prediction Markets Are Very Accurate](https://marginalrevolution.com/?p=91721) — Oct 2025
15. [BettingUSA: Weather Betting Guide](https://www.bettingusa.com/prediction-markets/weather/) — Nov 2025
16. [Colorado Sun: Prediction Markets on Climate](https://coloradosun.com/2026/03/10/prediction-market-money-wagers-climate-change-colorado/) — Mar 2026
17. [MEXC: Polymarket Arbitrage Strategies](https://www.mexc.co/news/584334) — distinct-baguette analysis
