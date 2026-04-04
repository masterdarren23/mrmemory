"""Community Reply Agent — Drafts helpful replies to relevant threads."""

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent.parent))

from core import ask_ollama, web_search, fetch_url, send_telegram, log_run, save_output, now_str
import config

SYSTEM_PROMPT = f"""You are a developer advocate for MrMemory who participates in online communities.
{config.PRODUCT_CONTEXT}
CRITICAL RULES:
- Be genuinely helpful FIRST. Answer their actual question.
- Only mention MrMemory if it's truly relevant to what they're asking.
- Sound like a real developer, not a marketer.
- If MrMemory isn't a good fit for their question, just help them anyway.
- Never be pushy or salesy."""


def run():
    print(f"[{now_str()}] Running Community Reply Agent...")

    queries = [
        "how to add memory to AI agent site:reddit.com",
        "LangChain memory problem site:reddit.com",
        "AI chatbot remember conversations site:stackoverflow.com",
        "agent memory between sessions site:github.com",
        "AI agent context window too small",
    ]

    all_results = []
    for q in queries:
        results = web_search(q, count=3)
        all_results.extend([r for r in results if "error" not in r])

    # Deduplicate
    seen = set()
    unique = []
    for r in all_results:
        if r["url"] not in seen:
            seen.add(r["url"])
            unique.append(r)

    if not unique:
        msg = "💬 Community Reply: No new threads found."
        send_telegram(msg)
        log_run("community_reply", msg)
        return msg

    # Try to fetch top threads for more context
    thread_context = ""
    for r in unique[:3]:
        text = fetch_url(r["url"], max_chars=2000)
        thread_context += f"\n### {r['title']}\nURL: {r['url']}\n{text[:1500]}\n"

    prompt = f"""Here are developer threads where someone needs help with AI agent memory (today is {now_str()}):

{thread_context}

For each thread, draft a reply that:
1. Directly answers their question or contributes value
2. Mentions MrMemory ONLY if it genuinely solves their problem
3. Includes a code example if relevant (pip install mrmemory, 3-line usage)
4. Sounds like a real developer sharing their experience
5. Is concise (max 150 words per reply)

Format:
### Thread: [title]
**Platform:** Reddit/SO/GitHub
**Their Problem:** One line
**Draft Reply:**
> Your reply

**Should we reply?** Yes/No (with reason)"""

    replies = ask_ollama(prompt, system=SYSTEM_PROMPT, temperature=0.6)

    header = f"# 💬 Community Replies — {now_str()}\n\n"
    full_report = header + replies
    path = save_output("community_reply", full_report)
    log_run("community_reply", replies)

    tg_msg = f"💬 *Community Reply Drafts*\n\n{replies[:3500]}"
    send_telegram(tg_msg)

    print(f"[{now_str()}] Community Reply complete. Saved to {path}")
    return replies


if __name__ == "__main__":
    config.load_env()
    run()
