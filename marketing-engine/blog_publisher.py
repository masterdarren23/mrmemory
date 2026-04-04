"""Blog Publisher — Converts markdown articles to HTML and publishes to mrmemory.dev/blog."""

import json
import os
import re
import sys
import io
import subprocess
import html
from datetime import datetime
from pathlib import Path

# Fix Windows encoding
if sys.stdout.encoding != "utf-8":
    sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding="utf-8", errors="replace")
    sys.stderr = io.TextIOWrapper(sys.stderr.buffer, encoding="utf-8", errors="replace")

LANDING_DIR = Path(r"C:\Users\johnl\.openclaw\workspace\amr-project\landing")
BLOG_DIR = LANDING_DIR / "blog"
POSTS_DIR = BLOG_DIR / "posts"
TEMPLATE_PATH = BLOG_DIR / "post-template.html"
INDEX_PATH = POSTS_DIR / "index.json"


def slugify(text: str) -> str:
    """Convert title to URL slug."""
    text = text.lower().strip()
    text = re.sub(r'[^\w\s-]', '', text)
    text = re.sub(r'[\s_]+', '-', text)
    text = re.sub(r'-+', '-', text)
    return text[:80].strip('-')


def markdown_to_html(md: str) -> str:
    """Convert markdown to HTML (simple converter, no dependencies)."""
    lines = md.split('\n')
    html_parts = []
    in_code_block = False
    in_list = False
    list_type = None
    
    for line in lines:
        # Code blocks
        if line.strip().startswith('```'):
            if in_code_block:
                html_parts.append('</code></pre>')
                in_code_block = False
            else:
                lang = line.strip()[3:].strip()
                html_parts.append(f'<pre><code class="language-{lang}">')
                in_code_block = True
            continue
        
        if in_code_block:
            html_parts.append(html.escape(line))
            continue
        
        stripped = line.strip()
        
        # Empty line
        if not stripped:
            if in_list:
                tag = 'ol' if list_type == 'ol' else 'ul'
                html_parts.append(f'</{tag}>')
                in_list = False
            html_parts.append('')
            continue
        
        # Headers
        if stripped.startswith('### '):
            html_parts.append(f'<h3>{_inline(stripped[4:])}</h3>')
            continue
        if stripped.startswith('## '):
            html_parts.append(f'<h2>{_inline(stripped[3:])}</h2>')
            continue
        if stripped.startswith('# '):
            # Skip h1 — we use the title from frontmatter
            continue
        
        # Unordered list
        if stripped.startswith('- ') or stripped.startswith('* '):
            if not in_list or list_type != 'ul':
                if in_list:
                    tag = 'ol' if list_type == 'ol' else 'ul'
                    html_parts.append(f'</{tag}>')
                html_parts.append('<ul>')
                in_list = True
                list_type = 'ul'
            html_parts.append(f'<li>{_inline(stripped[2:])}</li>')
            continue
        
        # Ordered list
        ol_match = re.match(r'^\d+\.\s+(.+)', stripped)
        if ol_match:
            if not in_list or list_type != 'ol':
                if in_list:
                    tag = 'ol' if list_type == 'ol' else 'ul'
                    html_parts.append(f'</{tag}>')
                html_parts.append('<ol>')
                in_list = True
                list_type = 'ol'
            html_parts.append(f'<li>{_inline(ol_match.group(1))}</li>')
            continue
        
        # Blockquote
        if stripped.startswith('> '):
            html_parts.append(f'<blockquote>{_inline(stripped[2:])}</blockquote>')
            continue
        
        # Table (basic)
        if '|' in stripped and stripped.startswith('|'):
            if stripped.replace('|', '').replace('-', '').replace(' ', '') == '':
                continue  # separator row
            cells = [c.strip() for c in stripped.split('|')[1:-1]]
            tag = 'th' if not any('<td>' in p for p in html_parts[-5:]) else 'td'
            row = ''.join(f'<{tag}>{_inline(c)}</{tag}>' for c in cells)
            if tag == 'th' and '<table>' not in '\n'.join(html_parts[-3:]):
                html_parts.append('<table>')
            html_parts.append(f'<tr>{row}</tr>')
            continue
        
        # Close list if needed
        if in_list:
            tag = 'ol' if list_type == 'ol' else 'ul'
            html_parts.append(f'</{tag}>')
            in_list = False
        
        # Paragraph
        html_parts.append(f'<p>{_inline(stripped)}</p>')
    
    if in_list:
        tag = 'ol' if list_type == 'ol' else 'ul'
        html_parts.append(f'</{tag}>')
    
    return '\n'.join(html_parts)


def _inline(text: str) -> str:
    """Convert inline markdown to HTML."""
    # Code
    text = re.sub(r'`([^`]+)`', r'<code>\1</code>', text)
    # Bold
    text = re.sub(r'\*\*([^*]+)\*\*', r'<strong>\1</strong>', text)
    # Italic
    text = re.sub(r'\*([^*]+)\*', r'<em>\1</em>', text)
    # Links
    text = re.sub(r'\[([^\]]+)\]\(([^)]+)\)', r'<a href="\2">\1</a>', text)
    return text


def parse_frontmatter(content: str) -> tuple[dict, str]:
    """Parse YAML-like frontmatter from markdown content."""
    meta = {}
    body = content
    
    if content.startswith('---'):
        parts = content.split('---', 2)
        if len(parts) >= 3:
            body = parts[2].strip()
            for line in parts[1].strip().split('\n'):
                if ':' in line:
                    key, val = line.split(':', 1)
                    key = key.strip().strip('"').strip("'")
                    val = val.strip().strip('"').strip("'")
                    # Handle arrays
                    if val.startswith('[') and val.endswith(']'):
                        val = [v.strip().strip('"').strip("'") for v in val[1:-1].split(',')]
                    meta[key] = val
    
    return meta, body


def publish_article(markdown_path: str) -> str | None:
    """Publish a markdown article to the blog. Returns the URL or None."""
    content = Path(markdown_path).read_text(encoding='utf-8')
    
    # Strip leading "Here is the article:" type preambles
    for prefix in ["Here is the article:", "Here's the article:", "Here is the blog post:"]:
        if content.startswith(prefix):
            content = content[len(prefix):].strip()
    
    meta, body = parse_frontmatter(content)
    
    title = meta.get('title', 'Untitled')
    description = meta.get('description', '')
    tags = meta.get('tags', [])
    date = meta.get('date', datetime.now().strftime('%Y-%m-%d'))
    
    if isinstance(tags, str):
        tags = [t.strip() for t in tags.split(',')]
    
    slug = slugify(title)
    
    # Convert markdown to HTML
    body_html = markdown_to_html(body)
    
    # Estimate read time
    word_count = len(body.split())
    read_time = f"{max(1, word_count // 200)} min read"
    
    # Build tags HTML
    tags_html = ''.join(f'<span class="tag">{t}</span>' for t in tags)
    
    # Load template
    template = TEMPLATE_PATH.read_text(encoding='utf-8')
    
    # Fill template
    page_html = template.replace('{{TITLE}}', html.escape(title))
    page_html = page_html.replace('{{DESCRIPTION}}', html.escape(description))
    page_html = page_html.replace('{{SLUG}}', slug)
    page_html = page_html.replace('{{DATE}}', date)
    page_html = page_html.replace('{{READ_TIME}}', read_time)
    page_html = page_html.replace('{{TAGS_HTML}}', tags_html)
    page_html = page_html.replace('{{CONTENT}}', body_html)
    
    # Write post HTML
    POSTS_DIR.mkdir(parents=True, exist_ok=True)
    post_path = POSTS_DIR / f"{slug}.html"
    post_path.write_text(page_html, encoding='utf-8')
    
    # Update index.json
    index = []
    if INDEX_PATH.exists():
        try:
            index = json.loads(INDEX_PATH.read_text(encoding='utf-8'))
        except Exception:
            index = []
    
    # Remove existing entry with same slug
    index = [p for p in index if p.get('slug') != slug]
    
    # Add new entry at top
    index.insert(0, {
        'slug': slug,
        'title': title,
        'description': description,
        'date': date,
        'readTime': read_time,
        'tags': tags,
    })
    
    INDEX_PATH.write_text(json.dumps(index, indent=2), encoding='utf-8')
    
    url = f"https://mrmemory.dev/blog/posts/{slug}.html"
    print(f"Published: {title} → {url}")
    return url


def git_push(message: str = "New blog post"):
    """Commit and push blog changes to GitHub."""
    cwd = str(LANDING_DIR.parent)
    subprocess.run(['git', 'add', 'landing/blog/'], cwd=cwd, capture_output=True)
    subprocess.run(['git', 'commit', '-m', message], cwd=cwd, capture_output=True)
    result = subprocess.run(['git', 'push', 'origin', 'master'], cwd=cwd, capture_output=True, text=True)
    if result.returncode == 0:
        print("Pushed to GitHub — Vercel will auto-deploy.")
        return True
    else:
        print(f"Git push failed: {result.stderr}")
        return False


def publish_and_deploy(markdown_path: str) -> str | None:
    """Full pipeline: markdown → HTML → git push → live on Vercel."""
    url = publish_article(markdown_path)
    if url:
        # Get title for commit message
        content = Path(markdown_path).read_text(encoding='utf-8')
        meta, _ = parse_frontmatter(content)
        title = meta.get('title', 'New post')
        git_push(f"Blog: {title}")
        return url
    return None


if __name__ == "__main__":
    import sys
    if len(sys.argv) < 2:
        print("Usage: python blog_publisher.py <markdown_file>")
        sys.exit(1)
    
    url = publish_and_deploy(sys.argv[1])
    if url:
        print(f"\n✅ Live at: {url}")
    else:
        print("\n❌ Failed to publish.")
