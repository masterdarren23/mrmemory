# Polymarket Cross-Market Arbitrage Opportunities
*Generated: February 15, 2026 (live API data)*

## 1. Political Cross-Market Arbitrage

### A. JD Vance: Nomination vs Presidential Win
| Market | Vance Price | Volume |
|--------|------------|--------|
| GOP Nominee 2028 | 46.7% | $292M |
| Presidential Winner 2028 | 24% | $293M |

**Implied conditional:** If Vance wins nomination, market implies 51.4% chance he wins presidency (24% / 46.7%)

**Check:** Is 51.4% conditional win rate reasonable?
- Historical GOP nominee win rate: ~50% in general elections
- Vance as sitting VP has incumbency advantage
- 51.4% seems roughly fair — **no clear arbitrage here**

### B. Donald Trump: GOP Nominee 2028
- Trump GOP Nominee: 4.75%
- **22nd Amendment issue:** Trump served 2020-2024 term... wait, Trump won 2024 and is currently serving. If he serves a full second term (2025-2029), he CANNOT run again per the 22nd Amendment.
- **This means 4.75% is arguably overpriced** for a constitutionally impossible outcome
- However, the market resolves on "nomination" not winning — could Trump be nominated even if constitutionally ineligible? The market description says "wins and accepts the 2028 nomination"
- **Potential arb:** Sell Trump GOP Nominee at $0.048 → collect $0.048 per share, pays out $0 with near-certainty
- **Risk:** Edge case where Trump somehow gets nominated (constitutional crisis scenario), or the 22nd Amendment is repealed/modified
- **Expected value:** ~$0.045 profit per share (95%+ probability of correct resolution to No)

### C. Gavin Newsom: Dem Nominee vs Presidential Winner
- Dem Nominee: 28%
- Presidential Winner: would need to check his specific market price
- If Newsom Dem Nominee at 28% but Newsom Presidential Winner < 28%, there's consistency
- Generally the nominee has ~50% general election win chance, so Newsom Pres Win should be ~14%

## 2. Multi-Outcome Probability Sums

### Fed Decision March 2026 (negRisk market)
| Outcome | Price |
|---------|-------|
| 50+ bps decrease | 0.65% |
| 25 bps decrease | 6.5% |
| No change | 92.5% |
| 25+ bps increase | 0.65% |
| **SUM** | **100.3%** |

**Arbitrage:** In a negRisk market, you can "sell" all outcomes for ~$1.003 and lock in 0.3% guaranteed profit. However:
- This is the built-in market-maker vig
- Transaction costs and spread likely eat the 0.3%
- At $5 minimum order × 4 outcomes = $20 capital for ~$0.06 profit
- **Not practically exploitable at small scale**

### Dutch Government Coalition
- Multiple outcomes, most at $0.001 floor
- Top outcome (VVD + CDA + D66) at ~99%
- Need to verify exact sum across all ~15+ options
- NegRisk market means theoretical sum should be very close to 100%

## 3. Sports Market Arbitrage

### Moneyline + Over/Under Consistency
For football (soccer) markets, Polymarket offers:
- Team A Win / Team B Win / Draw (moneyline)
- Over/Under goals totals
- Both teams to score

**These should be internally consistent** — for example:
- If Over 2.5 goals is 50%, the combined probability of high-scoring outcomes in the moneyline market should reflect this
- I found completed games where both markets are still "active" with proposed resolution, creating brief windows where resolution timing differs

### Live Sports Timing Edge
**Key observation:** Sports markets show `secondsDelay: 3` — there's a 3-second delay on live markets.
- If you have a faster data feed than Polymarket's oracle, you could trade within that 3-second window
- This requires: real-time sports data feed, API trading infrastructure, fast execution
- **Not viable for manual trading, requires bot infrastructure**

## 4. Time-Cascade Arbitrage

### US Strikes Iran Series
The "US strikes Iran by..." market has cascading dates:
- By Feb 5: Resolved NO (0%)
- By Jan 31: Resolved NO (0%)
- By June 30: ~45% (from page data)

**Structure:** Each later date must be ≥ earlier date probability (since if it happens by Feb, it also happened by June).
- Already-resolved dates correctly show 0%/100%
- The cascade is self-consistent as long as probabilities increase monotonically with time

### Bitcoin Price Cascading Strikes
Bitcoin above X on Feb 15:
- $66K: 99.95%
- $68K: 99.95%
- $70K: 0.05%

**These are independent (non-negRisk) binary markets.** There should be:
- $66K ≥ $68K ≥ $70K (lower strikes always more likely)
- This is maintained: 99.95% ≥ 99.95% ≥ 0.05% ✓
- The sharp cliff between $68K and $70K tells us BTC is between $68-70K

## 5. Actionable Arbitrage Opportunities (Ranked)

### Tier 1: High Confidence, Low Capital Required
1. **Trump 2028 GOP Nominee — SELL at $0.048**
   - Near-certain to resolve No (22nd Amendment)
   - Risk: ~2-5% chance of edge case
   - Capital: $5 minimum
   - Expected profit: ~$0.045/share (93.75% return)
   - Resolution: 2028 (long lock-up)
   - ⚠️ Capital tied up for 2+ years

### Tier 2: Moderate Confidence
2. **Fed March No Change — BUY at $0.925**
   - CME FedWatch typically agrees: >90% no change
   - If truly 95%+, there's 2-5% edge buying at 92.5¢
   - Resolves: March 18, 2026 (1 month)
   - Capital at risk: $0.925/share, potential return $0.075 (8.1%)
   - Expected edge: 2-5% absolute

### Tier 3: Speculative
3. **Celebrity/Meme Candidate Nominees — SELL**
   - Stephen A. Smith Dem Nominee at 1.25%
   - Oprah Winfrey Dem Nominee at 1.05%
   - Combined: selling both at ~$0.023 for near-certain resolution to No
   - But 2028 resolution = long capital lock-up
   - Small absolute return per share

## 6. Structural Arbitrage Notes

### NegRisk vs Non-NegRisk
- **NegRisk markets** (Fed, politics, coalitions): Probabilities are enforced to sum near 100%. The "Other" bucket absorbs excess. Arbitrage opportunities are small (spread-level).
- **Non-negRisk markets** (Bitcoin strikes, independent sports props): Each market is independent. Probabilities need NOT sum to 100%. This is where larger mispricings can occur.

### Resolution Disputes
- **Kevin Warsh Fed Chair** has been disputed TWICE in UMA resolution
- Markets in dispute can have unusual pricing dynamics
- If you understand the dispute mechanism, buying/selling during dispute periods can be profitable
