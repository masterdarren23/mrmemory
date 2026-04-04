"""GEO Agent — Generative Engine Optimization for AI search visibility."""

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent.parent))

from core import ask_ollama, web_search, send_telegram, log_run, save_output, now_str
import config

SYSTEM_PROMPT = f"""You are a Generative Engine Optimization (GEO) specialist.
{config.PRODUCT_CONTEXT}
GEO is about optimizing content so AI models (ChatGPT, Perplexity, Google AI Overviews) cite your product when users ask relevant questions.
You understand how LLMs retrieve and rank sources."""


def run():
    print(f"[{now_str()}] Running GEO Agent...")

    # Test what AI search surfaces for key queries
    test_queries = [
        "best memory API for AI agents",
        "how to add long term memory to LangChain",
        "Mem0 alternatives",
        "persistent memory for AI agents",
        "AI agent memory solutions comparison",
    ]

    search_results = {}
    for q in test_queries:
        search_results[q] = web_search(q, count=10)

    prompt = f"""Analyze MrMemory's visibility in AI-generated search results and recommend GEO improvements (today is {now_str()}).

When users ask AI assistants these questions, here's what currently appears in web results (which AI models use as sources):

{_format_all(search_results)}

Analyze and provide:

1. **Visibility Scorecard** — For each query, does mrmemory.dev appear? If not, who does?
2. **What Gets Cited** — What do the top-ranking pages have in common? (structure, authority signals, content depth)
3. **Content to Create** — Specific pages/articles that would get picked up by AI models:
   - Comparison pages ("MrMemory vs Mem0 vs Zep")
   - Tutorial pages ("How to add memory to LangChain agents")
   - FAQ pages with structured Q&A
4. **Schema/Structured Data** — JSON-LD or metadata to add to mrmemory.dev
5. **Authority Building** — How to get cited as a source (GitHub stars, Stack Overflow answers, Wikipedia mentions)
6. **3 Priority Actions** — Most impactful things to do this week

Remember: GEO is different from SEO. We're optimizing for AI citation, not just Google ranking."""

    analysis = ask_ollama(prompt, system=SYSTEM_PROMPT, temperature=0.4)

    header = f"# 🤖 GEO Report — {now_str()}\n\n"
    full_report = header + analysis
    path = save_output("geo", full_report)
    log_run("geo", analysis)

    tg_msg = f"🤖 *GEO Report*\n\n{analysis[:3500]}"
    send_telegram(tg_msg)

    print(f"[{now_str()}] GEO complete. Saved to {path}")
    return analysis


def _format_all(data: dict) -> str:
    lines = []
    for query, results in data.items():
        lines.append(f"\n### \"{query}\"")
        for i, r in enumerate(results):
            if "error" not in r:
                is_us = "⭐" if "mrmemory" in r.get("url", "").lower() else ""
                lines.append(f"  {i+1}. {r['title'][:80]} — {r['url'][:80]} {is_us}")
    return "\n".join(lines)


if __name__ == "__main__":
    config.load_env()
    run()
