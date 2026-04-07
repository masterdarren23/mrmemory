import requests

API_KEY = "uRut7ESDweUDGwXd5TPsQWPM"
HEADERS = {"api-key": API_KEY, "Content-Type": "application/json"}

# 1. Find and delete the broken article
print("=== Finding broken article ===")
r = requests.get("https://dev.to/api/articles/me/published", headers=HEADERS)
articles = r.json()
for a in articles:
    print(f"  {a['id']} | {a['title'][:60]}")
    if "scaling truth" in a["title"].lower():
        print(f"  -> Deleting broken article {a['id']}...")
        # Dev.to doesn't have DELETE, but we can unpublish
        r2 = requests.put(f"https://dev.to/api/articles/{a['id']}", headers=HEADERS, json={"article": {"published": False}})
        print(f"     Unpublished: {r2.status_code}")

# 2. Cross-post "Building a Chatbot That Remembers"
print("\n=== Cross-posting missing article ===")
# Read the HTML and extract content
from pathlib import Path
import re

html_path = Path(r"C:\Users\johnl\.openclaw\workspace\mrmemory\landing\blog\posts\building-a-chatbot-that-remembers-leveraging-mrmemory-for-ai-powered-conversatio.html")
html = html_path.read_text(encoding="utf-8")

# Extract article body content between <article> tags
match = re.search(r'<article[^>]*>(.*?)</article>', html, re.DOTALL)
if not match:
    print("Could not extract article content")
else:
    # We need markdown for Dev.to, let's construct it manually
    title = "Building a Chatbot That Remembers: Leveraging MrMemory for AI-Powered Conversations"
    canonical = "https://mrmemory.dev/blog/posts/building-a-chatbot-that-remembers-leveraging-mrmemory-for-ai-powered-conversatio.html"
    
    # Extract text content roughly
    content = match.group(1)
    # Strip HTML tags for a rough markdown version
    content = re.sub(r'<h2[^>]*>(.*?)</h2>', r'\n## \1\n', content)
    content = re.sub(r'<h3[^>]*>(.*?)</h3>', r'\n### \1\n', content)
    content = re.sub(r'<p[^>]*>(.*?)</p>', r'\1\n\n', content, flags=re.DOTALL)
    content = re.sub(r'<code[^>]*>(.*?)</code>', r'`\1`', content)
    content = re.sub(r'<pre[^>]*>(.*?)</pre>', r'```\n\1\n```\n', content, flags=re.DOTALL)
    content = re.sub(r'<li[^>]*>(.*?)</li>', r'- \1', content)
    content = re.sub(r'<a\s+href="([^"]*)"[^>]*>(.*?)</a>', r'[\2](\1)', content)
    content = re.sub(r'<strong>(.*?)</strong>', r'**\1**', content)
    content = re.sub(r'<em>(.*?)</em>', r'*\1*', content)
    content = re.sub(r'<[^>]+>', '', content)  # Strip remaining HTML
    content = re.sub(r'\n{3,}', '\n\n', content).strip()
    
    body = f"""---
title: "{title}"
published: true
tags: ["mrmemory", "ai", "chatbot", "langchain"]
canonical_url: "{canonical}"
---

{content}
"""
    
    r = requests.post("https://dev.to/api/articles", headers=HEADERS, json={"article": {"title": title, "body_markdown": body, "published": True, "tags": ["mrmemory", "ai", "chatbot", "langchain"], "canonical_url": canonical}})
    print(f"  Posted: {r.status_code}")
    if r.status_code < 300:
        print(f"  URL: {r.json().get('url')}")
    else:
        print(f"  Error: {r.text[:300]}")
