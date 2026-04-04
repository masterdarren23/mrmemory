"""Social Media Agent — Creates daily social media posts."""

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent.parent))

from core import ask_ollama, web_search, send_telegram, log_run, save_output, now_str, get_db
import config

SYSTEM_PROMPT = f"""You are a social media content creator for MrMemory, targeting AI developers.
{config.PRODUCT_CONTEXT}
Write like an indie developer building in public — not corporate marketing.
Posts should be punchy, technical enough to earn respect, but accessible.
Use developer humor when appropriate."""


def run():
    print(f"[{now_str()}] Running Social Media Agent...")

    # Get trending AI topics to hook into
    trends = web_search("AI agents news today 2026", count=5)
    dev_trends = web_search("LangChain CrewAI new features 2026", count=3)

    prompt = f"""Create 3 Twitter/X posts for MrMemory for today ({now_str()}).

Trending in AI right now:
{_format_results(trends)}

Developer framework news:
{_format_results(dev_trends)}

Create 3 different posts — mix these content types:
1. **Code snippet** — short Python or TypeScript example showing MrMemory (3-5 lines max in the tweet)
2. **Pain point** — relatable developer frustration about AI agents forgetting things
3. **Trend hook** — tie MrMemory into something trending in AI right now
4. **Build in public** — share a milestone, metric, or behind-the-scenes insight
5. **Comparison** — MrMemory vs building your own memory layer
6. **Meme/humor** — developer humor about AI context windows

For each post provide:
- The tweet text (max 280 chars, or mark as thread if longer)
- Best time to post (EST)
- 2-3 relevant hashtags
- Whether to include a link (and which one)

Make them feel authentic — not like marketing. A developer scrolling their feed should stop and think "huh, that's interesting."
"""

    posts = ask_ollama(prompt, system=SYSTEM_PROMPT, temperature=0.8)

    # Save drafts to DB
    db = get_db()
    db.execute(
        "INSERT INTO posts (platform, content, created_at, status) VALUES (?, ?, ?, ?)",
        ("twitter", posts, now_str(), "draft"),
    )
    db.commit()
    db.close()

    # Save output
    header = f"# 📱 Social Media Posts — {now_str()}\n\n"
    full_report = header + posts
    path = save_output("social_media", full_report)
    log_run("social_media", posts)

    # Extract individual posts and send for approval
    individual_posts = _extract_posts(posts)
    if individual_posts:
        from telegram_bot import send_post_for_approval
        send_telegram("📱 *Social Media Posts Ready* — approve below:")
        for post_text in individual_posts:
            send_post_for_approval(post_text, platform="X (@realmrmemory)")
            import time; time.sleep(1)
    else:
        tg_msg = f"📱 *Social Media Posts Ready*\n\n{posts[:3500]}"
        send_telegram(tg_msg)

    print(f"[{now_str()}] Social Media complete. Saved to {path}")
    return posts


def _format_results(results: list[dict]) -> str:
    lines = []
    for r in results:
        if "error" not in r:
            lines.append(f"- {r['title']}: {r['snippet'][:150]}")
    return "\n".join(lines) if lines else "No results found."


def _extract_posts(raw: str) -> list[str]:
    """Try to extract individual tweet texts from Ollama output."""
    posts = []
    lines = raw.split("\n")
    capture = False
    current = []
    
    for line in lines:
        # Look for tweet text markers
        lower = line.lower().strip()
        if any(marker in lower for marker in ["tweet text:", "tweet:", "post text:"]):
            if current:
                posts.append("\n".join(current).strip().strip('"').strip("'"))
            current = []
            # Check if text is on the same line after the colon
            parts = line.split(":", 1)
            if len(parts) > 1 and parts[1].strip():
                text = parts[1].strip().strip('"').strip("'")
                if text:
                    current.append(text)
            capture = True
        elif capture and lower.startswith(("best time", "hashtag", "link:", "**best", "**hashtag", "**link")):
            if current:
                posts.append("\n".join(current).strip().strip('"').strip("'"))
                current = []
            capture = False
        elif capture and line.strip():
            current.append(line.strip().strip('"').strip("'"))
    
    if current:
        posts.append("\n".join(current).strip().strip('"').strip("'"))
    
    # Filter out empty or too-short posts
    return [p for p in posts if len(p) > 20]


if __name__ == "__main__":
    config.load_env()
    run()
