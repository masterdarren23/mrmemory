"""CVR Optimization Agent — Landing page conversion rate optimization."""

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent.parent))

from core import ask_ollama, web_search, fetch_url, send_telegram, log_run, save_output, now_str
import config

SYSTEM_PROMPT = f"""You are a conversion rate optimization expert for developer tools SaaS.
{config.PRODUCT_CONTEXT}
You think in terms of developer psychology — what makes a dev trust a tool enough to try it.
Your recommendations are specific with exact copy suggestions."""


def run():
    print(f"[{now_str()}] Running CVR Optimization Agent...")

    # Fetch current pages
    landing = fetch_url("https://mrmemory.dev", max_chars=4000)
    docs = fetch_url("https://mrmemory.dev/docs", max_chars=3000)

    # Check competitor landing pages for comparison
    competitor = fetch_url("https://mem0.ai", max_chars=3000)

    prompt = f"""Analyze MrMemory's conversion funnel and recommend specific improvements (today is {now_str()}).

## Current Landing Page (mrmemory.dev)
{landing[:3000]}

## Current Docs Page (mrmemory.dev/docs)
{docs[:2000]}

## Competitor Landing Page (mem0.ai)
{competitor[:2000]}

Analyze the full funnel: Landing → Docs → GitHub → Stripe Trial Signup

Provide:
1. **Funnel Analysis** — Where are developers most likely dropping off and why?
2. **Hero Section** — Is the value prop clear in 5 seconds? Specific copy improvements.
3. **Social Proof** — What's missing? (testimonials, logos, GitHub stars badge, download counts)
4. **CTA Optimization** — Are buttons compelling? Exact text/placement changes.
5. **Trust Signals** — What would make a skeptical developer click "Start Trial"?
6. **Docs → Signup Bridge** — How to convert docs readers into trial users
7. **vs Competitor** — What does Mem0's page do better? What do we do better?
8. **3 Quick Wins** — Changes that can be made in under an hour
9. **A/B Test Ideas** — 2 tests ranked by expected impact

Give exact before/after copy for every recommendation."""

    analysis = ask_ollama(prompt, system=SYSTEM_PROMPT, temperature=0.5)

    header = f"# 🎯 CVR Optimization Report — {now_str()}\n\n"
    full_report = header + analysis
    path = save_output("cvr", full_report)
    log_run("cvr", analysis)

    tg_msg = f"🎯 *CVR Optimization Report*\n\n{analysis[:3500]}"
    send_telegram(tg_msg)

    print(f"[{now_str()}] CVR complete. Saved to {path}")
    return analysis


if __name__ == "__main__":
    config.load_env()
    run()
