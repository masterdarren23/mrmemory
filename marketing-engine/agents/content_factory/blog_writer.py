"""Blog Writer — Generates full SEO-optimized articles."""

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent.parent.parent))

from core import ask_ollama, web_search, fetch_url, send_telegram, log_run, save_output, now_str, get_db
import config

SYSTEM_PROMPT = f"""You are a technical content writer for MrMemory, writing for AI developers.
{config.PRODUCT_CONTEXT}

Writing style:
- Technical but accessible — a mid-level Python developer should follow easily
- Include real code examples (Python primarily, TypeScript when relevant)
- Use headers, bullet points, code blocks for scannability
- SEO-optimized: use the target keyword naturally in title, intro, headers, and conclusion
- 1200-2000 words per article
- Include a clear CTA at the end (try MrMemory, link to docs)
- Be genuinely informative — the article should be valuable even without MrMemory
- Mention MrMemory naturally as a solution, not as the entire focus"""


def write_article(topic: str) -> str:
    """Research a topic and write a full blog article."""
    print(f"  Researching: {topic}")
    
    # Research phase
    research = web_search(topic, count=8)
    
    # Fetch top 2 results for deeper context
    deep_research = ""
    for r in research[:2]:
        if "error" not in r:
            text = fetch_url(r["url"], max_chars=3000)
            deep_research += f"\n### Source: {r['title']}\n{text[:2000]}\n"
    
    prompt = f"""Write a comprehensive, SEO-optimized blog article.

TOPIC: {topic}

RESEARCH (use this for accuracy, don't copy):
{_format_results(research)}

{deep_research[:4000]}

Write the full article with:
1. An engaging title (with target keyword)
2. Meta description (155 chars max)
3. Introduction that hooks the reader (mention the pain point)
4. 3-5 main sections with H2 headers
5. Code examples where relevant (use MrMemory's actual API):
   - pip install mrmemory
   - from mrmemory import MrMemory
   - client = MrMemory(api_key="your-key")
   - client.remember("user prefers dark mode", tags=["preferences"])
   - results = client.recall("what theme does the user like?")
6. A comparison or alternatives section (mention Mem0, Zep, MemGPT fairly)
7. Conclusion with CTA to try MrMemory
8. Suggested internal links and tags

Format as proper markdown with frontmatter:
---
title: "..."
description: "..."
tags: [...]
date: {now_str()[:10]}
---"""

    article = ask_ollama(prompt, system=SYSTEM_PROMPT, temperature=0.6, max_tokens=8192)
    return article


def _format_results(results: list[dict]) -> str:
    lines = []
    for r in results:
        if "error" not in r:
            lines.append(f"- {r['title']}: {r['snippet'][:200]}")
    return "\n".join(lines) if lines else "No results."
