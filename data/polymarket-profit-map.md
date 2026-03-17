# Polymarket Profit Map: Where Is the Money?
*Generated: February 15, 2026 (live API data)*

## Executive Summary

The best opportunities on Polymarket for a $100 starting capital are:
1. **Near-certain resolution bets** (buying No on impossible outcomes) — slow but safe
2. **Daily crypto directional markets** — frequent but low liquidity
3. **News-driven political trading** — requires speed and knowledge
4. **Sports event trading** — high volume around game days

**15-minute crypto markets do NOT currently exist on Polymarket.** The shortest crypto cycles are daily.

---

## Ranked Opportunities

### 🥇 Rank 1: Fed March "No Change" — BUY at $0.925
| Metric | Value |
|--------|-------|
| **Ease of execution** | ⭐⭐⭐⭐⭐ Simple binary buy |
| **Capital efficiency** | ⭐⭐⭐⭐ 8.1% return if correct |
| **Expected ROI** | +2-5% edge (buy at 92.5¢, true prob ~95%) |
| **Time to resolution** | 31 days (March 18, 2026) |
| **Liquidity** | $1.1M (excellent) |
| **Fees** | None |

**Play:** Buy "No change" at $0.925. If Fed holds rates (overwhelming consensus), collect $1.00.
**With $100:** Buy ~108 shares at $0.925 each → if correct, receive $108.11 → **$8.11 profit (8.1%) in 31 days**
**Annualized: ~95% APR**
**Risk:** Fed unexpectedly cuts or raises. CME FedWatch and market consensus strongly favor no change.

---

### 🥈 Rank 2: Trump 2028 GOP Nominee — SELL (Buy "No" at $0.952)
| Metric | Value |
|--------|-------|
| **Ease of execution** | ⭐⭐⭐⭐⭐ |
| **Capital efficiency** | ⭐⭐⭐ 5% return |
| **Expected ROI** | ~4.5% near-certain profit |
| **Time to resolution** | ~2.5 years |
| **Liquidity** | $277K |

**Play:** Buy "No" on Trump winning 2028 GOP nomination at $0.952. He constitutionally cannot serve a third term.
**With $100:** Buy ~105 shares at $0.952 → collect $105 at resolution → **$5 profit, but locked for 2+ years**
**Annualized: ~2% APR** — not great, but nearly risk-free
**Risk:** Constitutional amendment, unusual interpretation

---

### 🥉 Rank 3: Daily Crypto Up/Down Markets
| Metric | Value |
|--------|-------|
| **Ease of execution** | ⭐⭐⭐⭐ One trade per day |
| **Capital efficiency** | ⭐⭐⭐⭐⭐ Small bets, quick resolution |
| **Expected ROI** | 5-15% per trade IF you have edge |
| **Time to resolution** | 24 hours |
| **Liquidity** | $17K (LOW) |

**Play:** When ETH or BTC "Down" is priced >65%, buy "Up" (contrarian/mean-reversion).
**With $100:** Bet $10-20 per day. At 29¢ for "Up", $20 buys ~69 shares → if correct, $69 payout = $49 profit.
**Key insight:** Over short periods, crypto direction is nearly random. If the market consistently overprices the trending direction, contrarian bets have +EV.
**Risk:** Low liquidity, crypto CAN trend for multiple days (momentum), not truly 50/50.

---

### Rank 4: Sports Pregame Markets
| Metric | Value |
|--------|-------|
| **Ease of execution** | ⭐⭐⭐ Requires sports knowledge |
| **Capital efficiency** | ⭐⭐⭐⭐ Multiple events daily |
| **Expected ROI** | 2-5% per bet with edge |
| **Time to resolution** | Hours to days |
| **Liquidity** | $100K-$1M per game |

**Play:** Find mispriced La Liga, Premier League, NBA markets. Compare Polymarket odds to professional sportsbook lines (Pinnacle, Betfair).
**With $100:** $10-25 per game, 4-8 games per week.
**Edge source:** Polymarket participants may be less sophisticated than sportsbook bettors — compare lines for discrepancies.

---

### Rank 5: Kevin Warsh Fed Chair — Resolution Dispute Play
| Metric | Value |
|--------|-------|
| **Ease of execution** | ⭐⭐ Complex, requires understanding UMA |
| **Capital efficiency** | ⭐⭐⭐⭐ 4.25% "No" side if wrong, $1 if right |
| **Expected ROI** | High if dispute reveals new info |
| **Time to resolution** | Unknown (end 2026 latest) |
| **Liquidity** | $604K |

**Play:** Monitor the UMA dispute process. If the dispute reveals Warsh won't be nominated, the 95.75% price crashes. "No" shares at $0.042 become worth $1 = 24x return.
**Risk:** Warsh IS actually nominated (most likely), you lose $0.042/share.

---

### Rank 6: US Strikes Iran — Volatility Trading
| Metric | Value |
|--------|-------|
| **Ease of execution** | ⭐⭐⭐ |
| **Capital efficiency** | ⭐⭐⭐⭐ High vol, low liq = big moves |
| **Expected ROI** | Highly variable |
| **Time to resolution** | Cascade dates through June 2026 |
| **Liquidity** | $1.4M (LOW for the volume) |

**Play:** Trade around news headlines. When tensions escalate, "Yes" prices spike. When they de-escalate, they drop. The low liquidity means small capital can catch big swings.
**With $100:** Buy "Yes" on escalation news, sell on de-escalation.
**Risk:** Unpredictable geopolitical events, illiquid exits.

---

## $100 Capital Deployment Strategy

### Conservative (Target: $120 in 60 days)
| Allocation | Amount | Strategy | Expected Return |
|-----------|--------|----------|----------------|
| Fed No Change March | $60 | Buy & hold to resolution | $4.86 (8.1%) |
| Daily Crypto Up/Down | $30 | $5-10/day contrarian bets | $3-15 variable |
| Reserve | $10 | Opportunity fund | — |
| **Total Expected** | | | **$8-20 profit** |

### Aggressive (Target: $200 in 60 days)
| Allocation | Amount | Strategy | Expected Return |
|-----------|--------|----------|----------------|
| Daily Crypto Up/Down | $40 | $10-20/day contrarian bets | $20-60 variable |
| Sports markets | $30 | $10/game, 2-3 games/week | $6-30 variable |
| Iran/Geopolitics | $20 | News-driven swing trading | $10-40 variable |
| Reserve | $10 | — | — |
| **Total Expected** | | | **$36-130 profit (high variance)** |

---

## Platform Mechanics to Know

### Fees
- **Trading fees:** NONE on most markets (feesEnabled: false)
- **Some markets have fees** — check `feesEnabled` field before trading
- **No withdrawal fees** beyond Polygon network gas (~$0.01)

### Order Types
- Limit orders only (no market orders in the traditional sense)
- Minimum order: $5
- Tick sizes: $0.001 (most markets), $0.01 (some)

### Settlement
- Binary outcomes: $1 per winning share, $0 per losing share
- NegRisk markets: shares can be merged/split
- UMA oracle resolution: typically 2-24 hours after event

### Key Risks
1. **Smart contract risk** — Polymarket runs on Polygon, uses CTF framework
2. **Oracle/UMA risk** — disputes can delay or change resolution (see Warsh)
3. **Liquidity risk** — many markets have thin books
4. **Regulatory risk** — Polymarket's legal status varies by jurisdiction
5. **Capital lock-up** — money is committed until resolution

---

## What's NOT on Polymarket (Things I Looked For)

1. ❌ **15-minute crypto prediction markets** — don't exist currently
2. ❌ **Crypto perpetual/rolling markets** — not available
3. ❌ **Leveraged markets** — all binary, no leverage
4. ❌ **Options-style markets** — binary only
5. ❌ **Cross-market combo bets** — must be placed individually

## Bottom Line

**Best single opportunity right now:** Fed March No Change at $0.925. Near-certain 8.1% return in 31 days with deep liquidity and no fees.

**Best recurring opportunity:** Daily crypto Up/Down markets for contrarian directional bets, if you can develop an edge on crypto short-term direction.

**Highest ceiling:** Geopolitical event trading (Iran, etc.) during news catalysts — large moves, low liquidity = big potential gains, but also big risks.
