"""SEO Agent — Keyword research and on-page optimization recommendations."""

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent.parent))

from core import ask_ollama, web_search, fetch_url, send_telegram, log_run, save_output, now_str
import config

SYSTEM_PROMPT = f"""You are an SEO specialist for MrMemory, a developer tools SaaS.
{config.PRODUCT_CONTEXT}
You focus on technical SEO for developer-facing products. Your recommendations are specific and implementable."""


def run():
    print(f"[{now_str()}] Running SEO Agent...")

    # Check current rankings
    ranking_queries = [
        "AI agent memory API",
        "persistent memory for AI agents",
        "LangChain memory solution",
        "Mem0 alternative",
        "AI agent long term memory",
    ]

    ranking_data = {}
    for q in ranking_queries:
        results = web_search(q, count=10)
        ranking_data[q] = results

    # Fetch current landing page for analysis
    landing_text = fetch_url("https://mrmemory.dev", max_chars=3000)

    prompt = f"""Analyze MrMemory's SEO position and recommend improvements (today is {now_str()}).

## Current Landing Page Content
{landing_text[:2000]}

## Search Rankings for Target Keywords
{_format_rankings(ranking_data)}

Provide:
1. **Keyword Scorecard** — for each target keyword, is mrmemory.dev appearing in top 10? Who's ranking instead?
2. **On-Page Fixes** — specific changes to mrmemory.dev meta tags, headings, content for better ranking
3. **Content Gaps** — blog posts or pages to create that would rank for valuable queries
4. **Technical SEO** — sitemap, robots.txt, structured data, page speed recommendations
5. **Backlink Opportunities** — sites linking to competitors that we could get links from
6. **3 Quick Wins** — things that can be done today for immediate SEO improvement

Be specific with exact title tags, meta descriptions, and heading suggestions."""

    analysis = ask_ollama(prompt, system=SYSTEM_PROMPT, temperature=0.4)

    header = f"# 🔍 SEO Report — {now_str()}\n\n"
    full_report = header + analysis
    path = save_output("seo", full_report)
    log_run("seo", analysis)

    tg_msg = f"🔍 *SEO Report*\n\n{analysis[:3500]}"
    send_telegram(tg_msg)

    print(f"[{now_str()}] SEO complete. Saved to {path}")
    return analysis


def _format_rankings(data: dict) -> str:
    lines = []
    for query, results in data.items():
        lines.append(f"\n### \"{query}\"")
        for i, r in enumerate(results):
            if "error" not in r:
                is_us = "✅ US" if "mrmemory" in r.get("url", "").lower() else ""
                lines.append(f"  {i+1}. {r['title'][:80]} — {r['url'][:80]} {is_us}")
    return "\n".join(lines)


if __name__ == "__main__":
    config.load_env()
    run()
