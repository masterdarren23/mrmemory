"""Competitor Watch Agent — Tracks Mem0, Zep, Letta for changes."""

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent.parent))

from core import ask_ollama, web_search, fetch_url, send_telegram, log_run, save_output, now_str
import config

SYSTEM_PROMPT = f"""You are a competitive intelligence analyst for MrMemory.
{config.PRODUCT_CONTEXT}
You track competitors and identify threats and opportunities. Be factual and strategic."""


def run():
    print(f"[{now_str()}] Running Competitor Watch Agent...")

    competitors = {
        "Mem0": [
            web_search("Mem0 ai agent memory news 2026", count=5),
            web_search("site:github.com mem0ai/mem0", count=3),
        ],
        "Zep": [
            web_search("Zep ai memory open source 2026", count=3),
        ],
        "Letta (MemGPT)": [
            web_search("Letta MemGPT agent memory 2026", count=3),
        ],
    }

    prompt = f"""Create a competitive intelligence report for MrMemory (today is {now_str()}).

## Competitor Data

"""
    for name, searches in competitors.items():
        prompt += f"### {name}\n"
        for results in searches:
            for r in results:
                if "error" not in r:
                    prompt += f"- {r['title']}: {r['snippet'][:200]}\n  {r['url']}\n"
        prompt += "\n"

    prompt += """Analyze and provide:
1. **Competitor Moves** — Any new features, funding, partnerships, or content from each competitor?
2. **Threat Assessment** — What should worry us? Rate each: 🔴 High / 🟡 Medium / 🟢 Low
3. **Feature Gap Analysis** — What do they have that we don't? What do we have that they don't?
4. **Positioning Opportunities** — How to differentiate MrMemory based on competitor weaknesses
5. **Counter-Moves** — 3 specific actions to take in response to competitor activity
6. **New Entrants** — Any new players entering the AI agent memory space?"""

    analysis = ask_ollama(prompt, system=SYSTEM_PROMPT, temperature=0.4)

    header = f"# 🕵️ Competitor Watch — {now_str()}\n\n"
    full_report = header + analysis
    path = save_output("competitor_watch", full_report)
    log_run("competitor_watch", analysis)

    tg_msg = f"🕵️ *Competitor Watch*\n\n{analysis[:3500]}"
    send_telegram(tg_msg)

    print(f"[{now_str()}] Competitor Watch complete. Saved to {path}")
    return analysis


if __name__ == "__main__":
    config.load_env()
    run()
