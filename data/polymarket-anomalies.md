# Polymarket Anomaly Detection Report
*Generated: February 15, 2026 (live API data)*

## 1. Probability Sum Anomalies (Multi-Outcome Markets)

### Fed Decision in March 2026
**Event Volume: $118M | Liquidity: $5.2M**
| Outcome | Price | Implied Prob |
|---------|-------|-------------|
| 50+ bps decrease | $0.007 | 0.65% |
| 25 bps decrease | $0.07 | 6.5% |
| No change | $0.925 | 92.5% |
| 25+ bps increase | $0.007 | 0.65% |
| **TOTAL** | | **100.3%** |

**Anomaly:** Sum exceeds 100% by ~0.3%. This is a negRisk market, so the overround represents the spread/vig built into prices. In theory, buying all outcomes costs $1.003, meaning a guaranteed 0.3% loss. This is normal for negRisk markets but worth noting: the vig is very thin here.

### Dutch Government Coalition
**Event Volume: $57M | Liquidity: $1.5M**
- VVD + CDA + D66: 99% (per Polymarket page)
- All other options: trading at $0.0005-$0.002
- Multiple outcomes sum to well under 100% based on visible prices (most at floor)
- **Anomaly:** The 99% favorite has $14.3M of its 24h volume concentrated in a single sub-market. This suggests wash trading or concentrated market-making activity.

## 2. Stale/Mispriced Markets

### Sports Markets Still Trading After Resolution
- **Rayo vs Atlético Madrid (Feb 14):** Game ended (score 3-0), period "VFT" (verified full time), but markets still showing as "active" and "not closed" with $5.9M volume. Atlético Madrid "win" at $0.001 — correctly reflecting the loss, but the market hasn't formally closed yet (UMA resolution "proposed"). **Volume is still occurring** ($4.9M in 24h) on a resolved event.
- **T1 vs BNK FEARX (LoL):** Shows 100% BNK FEARX, $5M vol. Already resolved but still listed as active.
- **Multiple La Liga, Bundesliga matches:** Several completed games with proposed resolution still trading at floor/ceiling prices with significant 24h volume.

**Opportunity:** These post-resolution markets sometimes have lag. If you can buy "No" shares at $0.001 on clearly resolved outcomes, you get 99.9¢ shares for 0.1¢ each once UMA finalizes. But minimum order is $5, and you'd need significant volume to make it worthwhile given the ~0.1% remaining price.

### Kevin Warsh Fed Chair Nomination
**Event Volume: $482M | Liquidity: $70M**
- Kevin Warsh: 95.75% ($0.957/$0.958 bid/ask)
- **CRITICAL:** UMA resolution status shows `["proposed", "disputed", "proposed", "disputed"]` — the resolution has been disputed TWICE
- Kevin Hassett: 0.25%
- **Anomaly:** A market trading at 95.75% with active resolution disputes. If Warsh isn't actually nominated yet, there's edge in the dispute outcome. The 4.25% "No" shares ($0.042) could be wildly underpriced if the dispute reveals something.

## 3. Known Information Contradictions

### Bitcoin Above $70K on Feb 15
- Price: $0.001 (0.05% Yes)
- **Current BTC price:** ~$68-69K range (based on the $68K market being at 99.95% Yes and $70K at 0.05%)
- This is correctly priced — BTC is below $70K at resolution time (noon ET Feb 15)
- The $66K and $68K markets both at 99.95% confirm BTC > $68K

### Ethereum Up or Down Feb 16
- Up: 29%, Down: 71%
- Volume: $440K, Liquidity: only $17K
- **Low liquidity relative to the bet** — significant slippage risk for any meaningful position
- The 71% "Down" pricing implies the market expects ETH to decline from its Feb 15 noon price to Feb 16 noon price

## 4. Overpriced Favorites / Underpriced Longshots

### Republican Presidential Nominee 2028
**Volume: $292M | Liquidity: $15M**
- J.D. Vance: 46.7% (bid 0.466 / ask 0.468, spread: 0.2%)
- Donald Trump: 4.75% (bid 0.047 / ask 0.048)
- Elise Stefanik: 0.85%
- **Note:** Trump at 4.75% for 2028 GOP nom seems LOW given his historical dominance. However, 22nd Amendment prohibits a third term (he won 2024), so this reflects potential constitutional issues. If anything, 4.75% is arguably OVERPRICED for an impossible outcome (Trump cannot run again after serving 2 terms).

### Democratic Presidential Nominee 2028
**Volume: $662M | Liquidity: $36M**
- Gavin Newsom: 28%
- Andy Beshear: 2.95%
- Gretchen Whitmer: 2.75%
- Stephen A. Smith: 1.25% ← A TV personality at 1.25% for Dem nomination
- Oprah Winfrey: 1.05%
- **Anomaly:** Entertainment/celebrity candidates (Smith, Oprah) combined ~2.3% seems like "meme pricing"

### Presidential Election Winner 2028
- JD Vance: 24%
- But Vance's GOP nomination is only 46.7%
- **Cross-check:** If Vance wins nomination AND election, his win probability should be ≤ his nomination probability. 24% < 46.7% ✓ (implies ~51% conditional win probability if nominated)

## 5. Volume/Liquidity Mismatches (Risk Indicators)

| Market | Total Vol | Liquidity | Vol/Liq Ratio |
|--------|-----------|-----------|---------------|
| Fed Chair | $482M | $70M | 6.9x |
| Dem Nominee 2028 | $662M | $36M | 18.4x |
| GOP Nominee 2028 | $292M | $15M | 19.5x |
| US Strikes Iran | $260M | $1.4M | **185x** |
| Dutch Coalition | $57M | $1.5M | 38x |

**US Strikes Iran** has extremely low liquidity ($1.4M) relative to volume ($260M). This means:
- Massive slippage for large orders
- Price can be moved significantly with relatively small capital
- The 45% probability for June 30 may be easy to push around

## 6. Fee Structure Observations
- Most major markets show `feesEnabled: false`
- The 15-minute crypto markets referenced in the task brief were NOT found in the current API data
- Daily crypto markets (Bitcoin up/down, ETH up/down) are the shortest-duration crypto markets currently active
- Order minimum: $5 across all markets
- Tick size: $0.001 for most markets, $0.01 for some competitive markets
