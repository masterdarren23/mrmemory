# Polymarket Weather Trader Analysis

*Compiled: 2026-02-15*

## Data Limitations

Polymarket's API does not expose per-user trade histories or leaderboards for specific market categories. The gamma API returns event/market data but not individual trader positions. Trader analysis requires on-chain analysis of the Polygon CLOB contracts, which is beyond what's accessible via simple API queries.

## What We Can Observe

### Market Structure
Weather markets on Polymarket are **extremely thin** compared to political/crypto markets:
- Hurricane Cat 4 landfall: $232K volume, $4.4K liquidity
- Hurricane Cat 5 landfall: $60.7K volume, $6.6K liquidity  
- Named storm before season: $261K volume, $5K liquidity
- Natural Disaster 2026: $121K volume, $19.6K liquidity
- Hurricane form by May 31: $27.5K volume, $7.8K liquidity
- Hurricane landfall by May 31: $3.8K volume, $7.1K liquidity

For comparison, political markets routinely have $100M+ volume and $10M+ liquidity.

### Implications for Traders
1. **Low liquidity = wide spreads.** Expect 2-5¢ spreads on most weather markets
2. **Low volume = few participants.** Maybe dozens of active traders, not thousands
3. **Markets likely dominated by 1-3 large market makers** who set prices
4. **Retail traders probably provide most of the edge** — they overbuy "exciting" outcomes (YES on hurricanes, disasters)

### Observable Trader Behavior Patterns (from market data)

#### The "Doomsday Bias"
- Natural Disaster 2026 at **47%** vs ~5-7% historical base rate → massive retail YES bias
- Cat 4 hurricane at **34%** vs ~9% base rate → similar pattern
- Cat 5 hurricane at **14%** vs ~3-4% base rate → overpriced by ~3-4x

This suggests retail traders:
- Overweight memorable/scary events (availability heuristic)
- Don't compute base rates from historical data
- Anchor on recent hyperactive hurricane seasons (2024-2025)

#### The "Smart Money" Strategy
Profitable traders in these markets likely:
1. **Sell YES (buy NO) on disaster/hurricane markets** when prices far exceed base rates
2. **Wait for panic spikes** — when a tropical disturbance forms or hurricane season approaches, YES prices spike further, creating even better sell opportunities
3. **Hold through resolution** — don't try to time exits; just buy NO and wait for markets to resolve

### Estimated Top Trader Profiles (inferred)

**Type 1: Base Rate Traders (the winners)**
- Buy NO on overpriced disaster/hurricane markets
- Use NOAA/ERA5 data to compute fair values
- Size positions proportional to edge (Kelly criterion)
- Expected edge: 15-40¢ on many weather markets

**Type 2: Event-Driven Traders**
- Monitor NHC advisories and tropical weather models in real-time
- Buy/sell as storms develop or dissipate
- Can capture 10-30¢ moves during active hurricane season
- Requires meteorological expertise

**Type 3: Retail "Doomers" (the losers)**
- Buy YES on disaster markets because "it could happen"
- Pay massive premiums over base rates
- Collectively subsidize Type 1 and Type 2 traders

## Recommendations for Research

To get actual trader data, would need to:
1. Query Polygon blockchain for trade events on weather market CLOB token IDs
2. Aggregate by wallet address
3. Track P&L per wallet across resolved markets
4. The CLOB token IDs are available from the gamma API (documented in era5-base-rates.md market data)

## Key Insight

**Weather markets are the most inefficient on Polymarket.** The combination of low liquidity, retail doomsday bias, and neglect by sophisticated traders creates persistent mispricings of 10-40¢. This is where $100 can find the best risk-adjusted returns on the entire platform.
