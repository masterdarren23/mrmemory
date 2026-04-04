"""Media Outreach Agent — Finds and drafts pitches to newsletters, podcasts, blogs."""

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent.parent))

from core import ask_ollama, web_search, send_telegram, log_run, save_output, now_str, get_db, was_contacted
import config

SYSTEM_PROMPT = f"""You are a PR and outreach specialist for MrMemory.
{config.PRODUCT_CONTEXT}
You write personalized, non-spammy outreach messages. Every pitch must reference something specific the target recently published.
You never send generic templates."""


def run():
    print(f"[{now_str()}] Running Media Outreach Agent...")

    targets = []
    searches = [
        "AI newsletter accepting submissions 2026",
        "AI developer tools blog post 2026",
        "AI podcast guest application developer tools",
        "AI agent tools review blog",
        "LangChain tutorial blog author",
        "best AI newsletters for developers",
    ]

    for query in searches:
        results = web_search(query, count=3)
        for r in results:
            if "error" not in r and not was_contacted(r.get("url", "")):
                targets.append(r)

    if not targets:
        msg = "📬 Media Outreach: No new targets found today."
        send_telegram(msg)
        log_run("media_outreach", msg)
        return msg

    # Deduplicate
    seen = set()
    unique = []
    for t in targets:
        if t["url"] not in seen:
            seen.add(t["url"])
            unique.append(t)

    prompt = f"""I found these potential outreach targets for MrMemory (today is {now_str()}):

{_format_results(unique)}

Pick the top 3 most promising targets and for each:
1. **Who they are** — name, platform, audience size estimate
2. **Why they're a fit** — what they cover that aligns with MrMemory
3. **Angle** — which story angle to pitch:
   - "Built entire SaaS with 6 AI agents in 3 weeks"
   - "Open-source Mem0 alternative at 1/1000th the funding"
   - "Why every AI agent needs persistent memory"
   - "From $0 to production SaaS in 3 weeks using AI coding agents"
4. **Draft pitch** — personalized email/DM (reference their recent work, max 150 words, clear CTA)

Make pitches feel personal and valuable to THEM — not just promotional for us."""

    outreach = ask_ollama(prompt, system=SYSTEM_PROMPT, temperature=0.6)

    # Track in DB
    db = get_db()
    for t in unique[:5]:
        try:
            db.execute(
                "INSERT OR IGNORE INTO contacts (name, platform, url, status) VALUES (?, ?, ?, ?)",
                (t["title"][:100], _guess_platform(t["url"]), t["url"], "identified"),
            )
        except Exception:
            pass
    db.commit()
    db.close()

    header = f"# 📬 Media Outreach — {now_str()}\n\n"
    full_report = header + outreach
    path = save_output("media_outreach", full_report)
    log_run("media_outreach", outreach)

    tg_msg = f"📬 *Media Outreach Targets*\n\n{outreach[:3500]}"
    send_telegram(tg_msg)

    print(f"[{now_str()}] Media Outreach complete. Saved to {path}")
    return outreach


def _format_results(results: list[dict]) -> str:
    lines = []
    for r in results:
        lines.append(f"- {r['title']}\n  URL: {r['url']}\n  {r['snippet'][:200]}")
    return "\n".join(lines)


def _guess_platform(url: str) -> str:
    if "substack" in url: return "Substack"
    if "youtube" in url: return "YouTube"
    if "twitter" in url or "x.com" in url: return "Twitter/X"
    if "linkedin" in url: return "LinkedIn"
    return "Web"


if __name__ == "__main__":
    config.load_env()
    run()
