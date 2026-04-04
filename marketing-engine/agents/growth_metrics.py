"""Growth Metrics Strategy Agent — Weekly analysis of all growth channels."""

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent.parent))

from core import ask_ollama, web_search, send_telegram, log_run, save_output, now_str
import config

SYSTEM_PROMPT = f"""You are a growth metrics analyst for MrMemory, a SaaS developer tool.
{config.PRODUCT_CONTEXT}
You analyze data and provide actionable growth recommendations. Be specific and data-driven.
Format your output in clean markdown with sections and bullet points."""


def run():
    print(f"[{now_str()}] Running Growth Metrics Agent...")

    # Gather data from multiple sources
    github_data = web_search("site:github.com masterdarren23/mrmemory", count=5)
    pypi_data = web_search("mrmemory pypi python package", count=3)
    npm_data = web_search("memorymr npm package", count=3)
    market_data = web_search("AI agent memory API market 2026", count=5)
    competitor_data = web_search("Mem0 ai agent memory latest news", count=5)

    prompt = f"""Analyze MrMemory's growth metrics and create a weekly strategy report.

Here's the data I gathered (today is {now_str()}):

## GitHub Intelligence
{_format_results(github_data)}

## PyPI (Python SDK)
{_format_results(pypi_data)}

## npm (TypeScript SDK)
{_format_results(npm_data)}

## Market Trends
{_format_results(market_data)}

## Competitor Activity (Mem0)
{_format_results(competitor_data)}

Create a report with:
1. **Key Metrics Summary** — what we know from available data
2. **Competitor Watch** — what Mem0 and others are doing
3. **Market Trends** — what's happening in AI agent memory space
4. **Top 3 Growth Actions** — specific, actionable things to do THIS week
5. **Content Ideas** — 3 blog/social post ideas based on trends
6. **Risk Flags** — anything concerning

Be specific. No fluff. Actionable recommendations only."""

    report = ask_ollama(prompt, system=SYSTEM_PROMPT, temperature=0.5)

    # Save and deliver
    header = f"# 📊 Weekly Growth Report — {now_str()}\n\n"
    full_report = header + report
    path = save_output("growth_metrics", full_report)
    log_run("growth_metrics", report)

    # Send to Telegram
    tg_msg = f"📊 *Weekly Growth Report*\n\n{report[:3500]}"
    send_telegram(tg_msg)

    print(f"[{now_str()}] Growth Metrics complete. Saved to {path}")
    return report


def _format_results(results: list[dict]) -> str:
    lines = []
    for r in results:
        if "error" in r:
            lines.append(f"- Error: {r['error']}")
        else:
            lines.append(f"- [{r['title']}]({r['url']})\n  {r['snippet']}")
    return "\n".join(lines) if lines else "No results found."


if __name__ == "__main__":
    config.load_env()
    run()
