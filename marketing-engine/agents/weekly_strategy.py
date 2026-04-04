"""Weekly Strategy Agent — Reads all agent outputs and creates unified plan."""

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent.parent))

from core import ask_ollama, send_telegram, log_run, save_output, now_str, get_db
import config

SYSTEM_PROMPT = f"""You are the chief marketing strategist for MrMemory.
{config.PRODUCT_CONTEXT}
You synthesize reports from all marketing agents into a unified weekly strategy.
Be decisive. Prioritize ruthlessly. Focus on what moves the needle."""


def run():
    print(f"[{now_str()}] Running Weekly Strategy Agent...")

    # Pull last week's agent outputs from DB
    db = get_db()
    rows = db.execute(
        "SELECT agent_name, run_at, output FROM agent_runs ORDER BY run_at DESC LIMIT 30"
    ).fetchall()
    db.close()

    summaries = ""
    for row in rows:
        summaries += f"\n### {row['agent_name']} ({row['run_at'][:10]})\n{row['output'][:500]}\n"

    if not summaries:
        summaries = "No agent runs found yet. This is the first week."

    prompt = f"""Create the weekly marketing strategy for MrMemory (today is {now_str()}).

Here are the reports from all marketing agents this week:
{summaries[:8000]}

Create a unified weekly plan:

1. **Week in Review** — What happened across all channels? Key wins and misses.
2. **Top 3 Priorities This Week** — The most important things to focus on, ranked
3. **Content Calendar** — What to post/publish each day this week
4. **Outreach Plan** — Who to contact and how
5. **Technical Tasks** — Any website/SEO/docs changes needed
6. **Metrics to Watch** — What numbers matter most right now
7. **Budget** — Any recommended spend (even small amounts)
8. **One Big Bet** — One bold move to try this week that could 10x visibility

Keep it actionable. Every item should have a clear owner (human or agent) and deadline."""

    strategy = ask_ollama(prompt, system=SYSTEM_PROMPT, temperature=0.5)

    header = f"# 🎯 Weekly Marketing Strategy — {now_str()}\n\n"
    full_report = header + strategy
    path = save_output("weekly_strategy", full_report)
    log_run("weekly_strategy", strategy)

    tg_msg = f"🎯 *Weekly Marketing Strategy*\n\n{strategy[:3500]}"
    send_telegram(tg_msg)

    print(f"[{now_str()}] Weekly Strategy complete. Saved to {path}")
    return strategy


if __name__ == "__main__":
    config.load_env()
    run()
