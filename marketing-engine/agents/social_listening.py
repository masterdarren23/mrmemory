"""Social Listening Agent — Monitors communities for relevant conversations."""

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent.parent))

from core import ask_ollama, web_search, send_telegram, log_run, save_output, now_str, get_db, was_mentioned
import config

SYSTEM_PROMPT = f"""You are a social listening specialist for MrMemory.
{config.PRODUCT_CONTEXT}
You find conversations where MrMemory could be naturally mentioned.
Your replies must be GENUINELY HELPFUL first — answer their question, then mention MrMemory naturally.
Never sound like spam. Sound like a developer who uses the product."""

SEARCH_QUERIES = [
    "AI agent memory site:reddit.com",
    "LangChain memory persistent site:reddit.com",
    "Mem0 alternative site:reddit.com",
    "AI agent forgets context site:reddit.com",
    "long term memory AI agent",
    "AI agent memory API",
    "LangChain memory solution 2026",
    "CrewAI memory persistent",
    "AI agent memory site:news.ycombinator.com",
    "AI agent memory site:twitter.com",
]


def run():
    print(f"[{now_str()}] Running Social Listening Agent...")

    all_results = []
    for query in SEARCH_QUERIES:
        results = web_search(query, count=3)
        for r in results:
            if "error" not in r and not was_mentioned(r.get("url", "")):
                all_results.append(r)

    if not all_results:
        msg = "🔇 Social Listening: No new relevant conversations found."
        send_telegram(msg)
        log_run("social_listening", msg)
        print(msg)
        return msg

    # Deduplicate by URL
    seen = set()
    unique = []
    for r in all_results:
        if r["url"] not in seen:
            seen.add(r["url"])
            unique.append(r)

    prompt = f"""I found these conversations that might be relevant to MrMemory (today is {now_str()}):

{_format_results(unique)}

For each conversation:
1. Rate urgency: 🔥 HOT (actively looking for solution), 🟡 WARM (discussing the problem), 🔵 COLD (tangentially related)
2. Write a suggested reply that is GENUINELY HELPFUL — answer their question/contribute to the discussion first, then mention MrMemory naturally if appropriate
3. If the conversation is too old or not a good fit, say SKIP

Only include the top 5 most relevant. Format as:

### [Platform] Title
**URL:** ...
**Urgency:** 🔥/🟡/🔵
**Context:** One line summary
**Suggested Reply:**
> Your reply here
"""

    analysis = ask_ollama(prompt, system=SYSTEM_PROMPT, temperature=0.5)

    # Track mentions in DB
    db = get_db()
    for r in unique[:10]:
        try:
            db.execute(
                "INSERT OR IGNORE INTO mentions (platform, url, title, found_at, urgency) VALUES (?, ?, ?, ?, ?)",
                (_guess_platform(r["url"]), r["url"], r["title"], now_str(), "warm"),
            )
        except Exception:
            pass
    db.commit()
    db.close()

    # Save and deliver
    header = f"# 👂 Social Listening Report — {now_str()}\n\n"
    full_report = header + analysis
    path = save_output("social_listening", full_report)
    log_run("social_listening", analysis)

    tg_msg = f"👂 *Social Listening Report*\n\n{analysis[:3500]}"
    send_telegram(tg_msg)

    print(f"[{now_str()}] Social Listening complete. Saved to {path}")
    return analysis


def _format_results(results: list[dict]) -> str:
    lines = []
    for r in results:
        platform = _guess_platform(r["url"])
        lines.append(f"- [{platform}] {r['title']}\n  URL: {r['url']}\n  {r['snippet']}")
    return "\n".join(lines)


def _guess_platform(url: str) -> str:
    if "reddit.com" in url: return "Reddit"
    if "news.ycombinator" in url: return "HN"
    if "twitter.com" in url or "x.com" in url: return "Twitter/X"
    if "github.com" in url: return "GitHub"
    return "Web"


if __name__ == "__main__":
    config.load_env()
    run()
