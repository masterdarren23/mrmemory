"""Comparison Page Writer — Generates detailed vs pages."""

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent.parent.parent))

from core import ask_ollama, web_search, fetch_url, now_str
import config

SYSTEM_PROMPT = f"""You are a technical writer creating fair, detailed comparison pages.
{config.PRODUCT_CONTEXT}

Comparison style:
- Be genuinely fair — acknowledge competitor strengths
- Use a feature comparison table
- Include pricing, ease of setup, and use case recommendations
- "Choose X if... Choose Y if..." format at the end
- MrMemory should win on facts, not spin
- Include actual code examples from both products
- 1000-1500 words"""

COMPARISONS = [
    {
        "competitor": "Mem0",
        "url": "https://mem0.ai",
        "search": "Mem0 AI agent memory features pricing 2026",
    },
    {
        "competitor": "Zep",
        "url": "https://www.getzep.com",
        "search": "Zep AI memory open source features 2026",
    },
    {
        "competitor": "Letta (MemGPT)",
        "url": "https://www.letta.com",
        "search": "Letta MemGPT agent memory features 2026",
    },
]


def write_comparison(competitor_info: dict) -> str:
    """Research a competitor and write a comparison page."""
    name = competitor_info["competitor"]
    print(f"  Researching {name}...")
    
    search_results = web_search(competitor_info["search"], count=5)
    competitor_page = fetch_url(competitor_info["url"], max_chars=3000)
    
    prompt = f"""Write a detailed comparison: MrMemory vs {name}

## Research on {name}
Website content: {competitor_page[:2000]}

Web results:
{_format(search_results)}

Write the comparison with:
1. **Title**: "MrMemory vs {name}: Detailed Comparison (2026)"
2. **Meta description** (155 chars)
3. **Introduction** — what both tools do, who they're for
4. **Feature Comparison Table** (markdown table):
   - Semantic search, compression, self-edit, governance, WebSocket, pricing, hosting, SDKs, frameworks supported
5. **Setup Comparison** — code examples for both (install + basic usage)
6. **Pricing Comparison** — detailed breakdown
7. **Use Case Recommendations** — "Choose MrMemory if... Choose {name} if..."
8. **Verdict**

Be fair. If {name} is better at something, say so."""

    return ask_ollama(prompt, system=SYSTEM_PROMPT, temperature=0.4, max_tokens=8192)


def _format(results):
    return "\n".join(f"- {r['title']}: {r['snippet'][:200]}" for r in results if "error" not in r)
