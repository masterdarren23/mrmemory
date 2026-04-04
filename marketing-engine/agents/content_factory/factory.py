"""Content Factory — Runs continuously, producing content every 30 minutes."""

import sys
import time
import random
import traceback
from pathlib import Path
from datetime import datetime

sys.path.insert(0, str(Path(__file__).parent.parent.parent))

from core import ask_ollama, send_telegram, log_run, save_output, now_str, get_db
import config
config.load_env()

from agents.content_factory.blog_writer import write_article
from agents.content_factory.answer_writer import find_and_draft_answers
from agents.content_factory.comparison_writer import write_comparison, COMPARISONS
from agents.content_factory.topics import SEO_BLOG_TOPICS


def get_next_topic(db) -> str | None:
    """Get the next unwritten topic."""
    written = set()
    rows = db.execute(
        "SELECT output FROM agent_runs WHERE agent_name = 'content_factory_blog' ORDER BY run_at DESC LIMIT 100"
    ).fetchall()
    for row in rows:
        for topic in SEO_BLOG_TOPICS:
            if topic.lower() in (row["output"] or "").lower():
                written.add(topic)
    
    remaining = [t for t in SEO_BLOG_TOPICS if t not in written]
    if remaining:
        return remaining[0]
    return None


def get_next_comparison(db) -> dict | None:
    """Get the next unwritten comparison."""
    rows = db.execute(
        "SELECT output FROM agent_runs WHERE agent_name = 'content_factory_comparison'"
    ).fetchall()
    done = set()
    for row in rows:
        for comp in COMPARISONS:
            if comp["competitor"].lower() in (row["output"] or "").lower():
                done.add(comp["competitor"])
    
    remaining = [c for c in COMPARISONS if c["competitor"] not in done]
    return remaining[0] if remaining else None


def run_cycle():
    """Run one production cycle — pick a task type and execute it."""
    db = get_db()
    
    # Count what we've done today
    today = datetime.now().strftime("%Y-%m-%d")
    today_count = db.execute(
        "SELECT COUNT(*) as c FROM agent_runs WHERE agent_name LIKE 'content_factory%' AND run_at LIKE ?",
        (f"{today}%",)
    ).fetchone()["c"]
    
    db.close()

    # Cycle through content types
    cycle_order = ["blog", "answers", "comparison", "blog", "answers", "blog"]
    task_type = cycle_order[today_count % len(cycle_order)]
    
    print(f"\n{'='*60}")
    print(f"[{now_str()}] Content Factory — Cycle #{today_count + 1} today")
    print(f"Task type: {task_type}")
    print('='*60)

    try:
        if task_type == "blog":
            _run_blog()
        elif task_type == "answers":
            _run_answers()
        elif task_type == "comparison":
            _run_comparison()
    except Exception as e:
        error_msg = f"Content Factory error: {e}\n{traceback.format_exc()}"
        print(error_msg)
        log_run("content_factory_error", error_msg, status="error")


def _run_blog():
    """Write one blog article."""
    db = get_db()
    topic = get_next_topic(db)
    db.close()
    
    if not topic:
        print("All blog topics written! Generating new topics...")
        topic = _discover_new_topic()
        if not topic:
            print("No new topics found. Skipping.")
            return
    
    print(f"Writing article: {topic}")
    article = write_article(topic)
    
    # Save
    path = save_output("content_factory", article)
    log_run("content_factory_blog", f"TOPIC: {topic}\n\n{article[:500]}")
    
    # Auto-publish blog posts directly
    from blog_publisher import publish_and_deploy
    url = publish_and_deploy(path)
    if url:
        send_telegram(f"📝 *Blog Post Published*\n\n*{topic}*\n\n{url}")
    else:
        send_telegram(f"📝 *Blog Post Failed to Publish*\n\n*{topic}*\n\nSaved locally: {path}")
    
    print(f"Article complete: {topic}")


def _run_answers():
    """Find questions and draft answers."""
    print("Finding questions to answer...")
    answers = find_and_draft_answers()
    
    if not answers:
        print("No new questions found.")
        return
    
    # Save all answers
    report = "# Developer Q&A Drafts\n\n"
    for a in answers:
        report += f"## {a['question']}\n**URL:** {a['url']}\n\n**Draft Answer:**\n{a['answer']}\n\n---\n\n"
    
    path = save_output("content_factory", report)
    log_run("content_factory_answers", f"Found {len(answers)} questions\n\n{report[:500]}")
    
    # Send each answer for review — user replies 👍 to approve
    from telegram_bot import send_qa_for_approval
    for a in answers:
        send_qa_for_approval(a["question"], a["url"], a["answer"])
        time.sleep(1)
    
    print(f"Drafted {len(answers)} answers.")


def _run_comparison():
    """Write a comparison page."""
    db = get_db()
    comp = get_next_comparison(db)
    db.close()
    
    if not comp:
        print("All comparison pages written.")
        return
    
    print(f"Writing comparison: MrMemory vs {comp['competitor']}")
    article = write_comparison(comp)
    
    path = save_output("content_factory", article)
    log_run("content_factory_comparison", f"COMPETITOR: {comp['competitor']}\n\n{article[:500]}")
    
    send_telegram(f"⚔️ *New Comparison Page*\n\nMrMemory vs {comp['competitor']}\n\n{article[:800]}\n\n_Full article: {path}_")
    
    print(f"Comparison complete: vs {comp['competitor']}")


def _discover_new_topic():
    """Use web search to find a new trending topic to write about."""
    from core import web_search
    results = web_search("AI agent memory problem developer 2026", count=5)
    
    if not results or "error" in results[0]:
        return None
    
    topics_text = "\n".join(f"- {r['title']}: {r['snippet'][:100]}" for r in results if "error" not in r)
    
    prompt = f"""Based on these trending topics in AI agent memory:

{topics_text}

Suggest ONE specific blog post title that would rank well and be useful to developers.
Just the title, nothing else. Make it specific and actionable."""

    topic = ask_ollama(prompt, temperature=0.8, max_tokens=100)
    return topic.strip().strip('"').strip("'")


def run_continuous(interval_minutes: int = 30):
    """Run the content factory continuously."""
    print(f"Content Factory starting — producing content every {interval_minutes} minutes")
    print(f"Topics queued: {len(SEO_BLOG_TOPICS)} blog posts, {len(COMPARISONS)} comparisons")
    print(f"Press Ctrl+C to stop\n")
    
    send_telegram(f"🏭 *Content Factory Online*\nProducing content every {interval_minutes} minutes.\nTopics queued: {len(SEO_BLOG_TOPICS)} blog posts, {len(COMPARISONS)} comparisons")
    
    while True:
        try:
            run_cycle()
            
            # Random interval to seem natural
            wait = interval_minutes * 60 + random.randint(-300, 300)
            wait = max(600, wait)  # Minimum 10 minutes
            print(f"\nNext cycle in {wait // 60} minutes...")
            time.sleep(wait)
            
        except KeyboardInterrupt:
            print("\nContent Factory shutting down.")
            send_telegram("🏭 Content Factory stopped.")
            break
        except Exception as e:
            print(f"Unexpected error: {e}")
            time.sleep(60)


if __name__ == "__main__":
    import argparse
    parser = argparse.ArgumentParser()
    parser.add_argument("--interval", type=int, default=30, help="Minutes between content cycles")
    parser.add_argument("--once", action="store_true", help="Run one cycle then exit")
    args = parser.parse_args()
    
    if args.once:
        run_cycle()
    else:
        run_continuous(args.interval)
