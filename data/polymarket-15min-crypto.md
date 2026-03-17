# Polymarket 15-Minute Crypto Trading Cycles
*Generated: February 15, 2026 (live API data)*

## Critical Finding: 15-Minute Markets NOT Currently Active

After extensive API searches across:
- `gamma-api.polymarket.com/markets` (slug searches for "15-min", "5-minute", "minute")
- `gamma-api.polymarket.com/series` (BTC/ETH minute series)
- Polymarket.com "Live Crypto" category
- All open markets sorted by volume

**I found NO active 15-minute crypto prediction markets on Polymarket as of Feb 15, 2026.**

Polymarket may have offered these in the past or may launch them in the future, but they are not currently available via the API.

## What IS Available: Short-Duration Crypto Markets

### 1. Daily "Up or Down" Markets
The shortest-cycle crypto markets currently active:

#### Ethereum Up or Down on February 16
- **Slug:** `ethereum-up-or-down-on-february-16`
- **Prices:** Up 29% / Down 71%
- **Volume:** $440K (total), active trading
- **Liquidity:** $17.3K (LOW)
- **Fees:** `feesEnabled: false`
- **Resolution:** Based on Binance ETH/USDT 1-minute candle at noon ET
- **Cycle:** New market created every day, resolves next day at noon ET
- **Structure:** Compares today's noon ET close to tomorrow's noon ET close

#### Bitcoin Up or Down (Similar structure, new daily)
- Same pattern: daily binary on whether BTC goes up or down
- Resolution via Binance BTC/USDT noon ET candle

### 2. Weekly "Bitcoin Above ___" Markets
Currently active: Bitcoin above $X on February 15:
| Strike | Price | Volume | Liquidity |
|--------|-------|--------|-----------|
| $66,000 | 99.95% | $336K | $414K |
| $68,000 | 99.95% | $544K | $440K |
| $70,000 | 0.05% | $884K | $499K |
| $78,000+ | ~0% | varies | varies |

- **Total event volume:** $4.8M
- **Total event liquidity:** $4.3M
- Created weekly on Saturdays, resolve following Saturday at noon ET
- Not negRisk — independent binary markets

### 3. Monthly "What Price Will Bitcoin Hit"
- **Volume:** $64M total, $3M/day
- **Liquidity:** $6M
- Leading option: ↑ $75,000 at 35%
- Resolves end of February

## Analysis: Trading the Daily Crypto Markets

### Structure
- Each market is a binary Yes/No (or Up/Down)
- Minimum order: $5
- Tick size: $0.001
- No fees on most crypto markets (`feesEnabled: false`)
- Resolution via Binance spot prices at specific candle times

### Edge Opportunities

#### A. Implied Volatility vs Realized Volatility
- ETH "Up or Down" at 29/71 implies the market thinks there's a 71% chance ETH goes DOWN tomorrow
- If you believe the true probability is closer to 50/50, buying "Up" at $0.29 offers significant edge
- Key question: does the market consistently misprice direction?

#### B. Momentum/Mean-Reversion Patterns
- After a down day, the market may overprice the next "Down" outcome (momentum bias)
- Contrarian play: buy "Up" after down days when the market prices "Down" too aggressively
- Would need historical data on daily crypto market pricing vs outcomes to validate

#### C. Liquidity Exploitation
- ETH Up/Down has only $17K liquidity
- A $500 market order could significantly move the price
- This creates both risk (slippage on YOUR orders) and opportunity (front-running large orders)

### Practical Constraints
1. **Minimum order $5** — fine for $100 starting capital
2. **24-hour cycle** — only one trade per day per market, not high-frequency
3. **Low liquidity** — limits position size before moving the market
4. **No fees** — this is the good news: pure spread cost only

### Strategy: Daily Crypto Binary Trading
**Capital required:** $100
**Approach:** 
1. Track BTC and ETH daily close at noon ET
2. When market prices Up/Down at >65% for either side, bet the underdog
3. Target: mean-reversion bias — crypto direction is inherently ~50/50 over short periods
4. Risk: $5-20 per trade, potential return: 2-3x on each winning trade
5. Expected edge: 5-15% per trade if mean-reversion hypothesis holds

### Why 15-Minute Markets Would Be Better (If They Existed)
- Higher frequency = more opportunities per day
- Shorter holding period = less risk per position
- More price discovery = more mispricings to exploit
- Fees would likely be enabled (reducing edge)

## Fee Structure (Current Crypto Markets)
- **Trading fees:** NONE on current crypto markets (`feesEnabled: false`)
- **Spread cost:** $0.001-$0.01 depending on market
- **Resolution cost:** None to holders
- **Gas/network:** Polymarket uses Polygon — negligible gas costs
- **Deposit/withdraw:** Standard Polygon bridging costs

## Recommendation
The daily crypto Up/Down markets are the closest available proxy for high-frequency crypto trading on Polymarket. The extremely low liquidity ($17K for ETH) limits scalability but the zero-fee structure and $5 minimum make them viable for small capital experimentation. Monitor Polymarket for any launch of intraday/minute-level crypto markets.
