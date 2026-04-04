"""Answer Writer — Drafts helpful answers to developer questions found online."""

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent.parent.parent))

from core import ask_ollama, web_search, fetch_url, send_telegram, log_run, now_str
import config

SYSTEM_PROMPT = f"""You are a helpful AI developer who uses MrMemory in your projects.
{config.PRODUCT_CONTEXT}

When answering questions:
- ALWAYS answer the actual question first with a genuine, helpful solution
- Only mention MrMemory if it's truly relevant to their problem
- Include working code examples
- Be humble — don't oversell
- Sound like a real developer sharing experience, not marketing
- Keep answers concise (150-300 words max)
- If their question has nothing to do with memory, just help them without mentioning MrMemory"""


def find_and_draft_answers() -> list[dict]:
    """Find unanswered questions and draft helpful responses."""
    queries = [
        "how to persist memory langchain agent site:stackoverflow.com",
        "AI agent remember conversations between sessions",
        "langchain memory database site:github.com discussions",
        "crewai agent memory persist site:reddit.com",
        "AI chatbot long term memory python",
    ]
    
    questions = []
    for query in queries:
        results = web_search(query, count=3)
        for r in results:
            if "error" not in r:
                questions.append(r)
    
    if not questions:
        return []
    
    answers = []
    for q in questions[:5]:  # Max 5 per run
        print(f"  Drafting answer for: {q['title'][:60]}")
        
        # Get more context from the page
        page_text = fetch_url(q["url"], max_chars=3000)
        
        prompt = f"""Someone asked this question:

Title: {q['title']}
URL: {q['url']}
Context: {page_text[:2000]}

Draft a helpful answer. Remember:
1. Answer their ACTUAL question first
2. Include working code if relevant
3. Only mention MrMemory if it genuinely helps solve their problem
4. Be concise (150-300 words)
5. Sound like a real developer, not a marketer"""

        answer = ask_ollama(prompt, system=SYSTEM_PROMPT, temperature=0.5)
        answers.append({
            "question": q["title"],
            "url": q["url"],
            "answer": answer,
        })
    
    return answers
