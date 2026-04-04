"""Core utilities: Ollama, Brave Search, Telegram, SQLite tracking."""

import json
import os
import sys
import sqlite3
import time
from datetime import datetime, timezone
from pathlib import Path

import requests

import config

# ---------------------------------------------------------------------------
# Ollama
# ---------------------------------------------------------------------------

def ask_ollama(prompt: str, system: str = "", temperature: float = 0.7, max_tokens: int = 4096) -> str:
    """Send a prompt to the local Ollama instance and return the response."""
    messages = []
    if system:
        messages.append({"role": "system", "content": system})
    messages.append({"role": "user", "content": prompt})

    try:
        resp = requests.post(
            f"{config.OLLAMA_URL}/api/chat",
            json={
                "model": config.OLLAMA_MODEL,
                "messages": messages,
                "stream": False,
                "options": {
                    "temperature": temperature,
                    "num_predict": max_tokens,
                },
            },
            timeout=300,
        )
        resp.raise_for_status()
        return resp.json()["message"]["content"]
    except Exception as e:
        return f"[OLLAMA ERROR] {e}"

# ---------------------------------------------------------------------------
# Brave Search
# ---------------------------------------------------------------------------

def web_search(query: str, count: int = 5) -> list[dict]:
    """Search the web via Brave Search API. Returns list of {title, url, snippet}."""
    config.load_env()
    if not config.BRAVE_API_KEY:
        return [{"error": "No BRAVE_API_KEY set"}]

    try:
        resp = requests.get(
            "https://api.search.brave.com/res/v1/web/search",
            headers={"X-Subscription-Token": config.BRAVE_API_KEY},
            params={"q": query, "count": count},
            timeout=15,
        )
        resp.raise_for_status()
        results = []
        for item in resp.json().get("web", {}).get("results", []):
            results.append({
                "title": item.get("title", ""),
                "url": item.get("url", ""),
                "snippet": item.get("description", ""),
            })
        return results
    except Exception as e:
        return [{"error": str(e)}]


def fetch_url(url: str, max_chars: int = 5000) -> str:
    """Fetch a URL and return the text content (truncated)."""
    try:
        resp = requests.get(url, timeout=15, headers={
            "User-Agent": "Mozilla/5.0 (compatible; MrMemoryBot/1.0)"
        })
        resp.raise_for_status()
        from bs4 import BeautifulSoup
        soup = BeautifulSoup(resp.text, "html.parser")
        # Remove script/style
        for tag in soup(["script", "style", "nav", "footer", "header"]):
            tag.decompose()
        text = soup.get_text(separator="\n", strip=True)
        return text[:max_chars]
    except Exception as e:
        return f"[FETCH ERROR] {e}"

# ---------------------------------------------------------------------------
# Telegram
# ---------------------------------------------------------------------------

def send_telegram(message: str, parse_mode: str = "Markdown") -> bool:
    """Send a message to the configured Telegram chat."""
    config.load_env()
    if not config.TELEGRAM_BOT_TOKEN:
        print(f"[TELEGRAM] No token set. Message:\n{message[:200]}...")
        return False

    try:
        # Split long messages (Telegram limit is 4096 chars)
        chunks = [message[i:i+4000] for i in range(0, len(message), 4000)]
        for chunk in chunks:
            resp = requests.post(
                f"https://api.telegram.org/bot{config.TELEGRAM_BOT_TOKEN}/sendMessage",
                json={
                    "chat_id": config.TELEGRAM_CHAT_ID,
                    "text": chunk,
                    "parse_mode": parse_mode,
                },
                timeout=15,
            )
            if not resp.ok:
                # Retry without parse_mode if markdown fails
                requests.post(
                    f"https://api.telegram.org/bot{config.TELEGRAM_BOT_TOKEN}/sendMessage",
                    json={
                        "chat_id": config.TELEGRAM_CHAT_ID,
                        "text": chunk,
                    },
                    timeout=15,
                )
            time.sleep(0.5)
        return True
    except Exception as e:
        print(f"[TELEGRAM ERROR] {e}")
        return False

# ---------------------------------------------------------------------------
# SQLite tracking
# ---------------------------------------------------------------------------

def get_db() -> sqlite3.Connection:
    """Get a connection to the tracking database."""
    db_path = Path(__file__).parent / config.DB_PATH
    db_path.parent.mkdir(parents=True, exist_ok=True)
    conn = sqlite3.connect(str(db_path))
    conn.row_factory = sqlite3.Row
    _init_db(conn)
    return conn


def _init_db(conn: sqlite3.Connection):
    """Create tables if they don't exist."""
    conn.executescript("""
        CREATE TABLE IF NOT EXISTS agent_runs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            agent_name TEXT NOT NULL,
            run_at TEXT NOT NULL,
            output TEXT,
            status TEXT DEFAULT 'ok'
        );
        CREATE TABLE IF NOT EXISTS contacts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT,
            platform TEXT,
            url TEXT UNIQUE,
            contacted_at TEXT,
            status TEXT DEFAULT 'pending'
        );
        CREATE TABLE IF NOT EXISTS posts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            platform TEXT,
            content TEXT,
            created_at TEXT,
            posted_at TEXT,
            status TEXT DEFAULT 'draft'
        );
        CREATE TABLE IF NOT EXISTS mentions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            platform TEXT,
            url TEXT UNIQUE,
            title TEXT,
            found_at TEXT,
            replied INTEGER DEFAULT 0,
            urgency TEXT DEFAULT 'warm'
        );
    """)
    conn.commit()


def log_run(agent_name: str, output: str, status: str = "ok"):
    """Log an agent run to the database."""
    db = get_db()
    db.execute(
        "INSERT INTO agent_runs (agent_name, run_at, output, status) VALUES (?, ?, ?, ?)",
        (agent_name, datetime.now(timezone.utc).isoformat(), output[:10000], status),
    )
    db.commit()
    db.close()


def was_contacted(url: str) -> bool:
    """Check if a URL has already been contacted."""
    db = get_db()
    row = db.execute("SELECT 1 FROM contacts WHERE url = ?", (url,)).fetchone()
    db.close()
    return row is not None


def was_mentioned(url: str) -> bool:
    """Check if a mention URL has already been tracked."""
    db = get_db()
    row = db.execute("SELECT 1 FROM mentions WHERE url = ?", (url,)).fetchone()
    db.close()
    return row is not None


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def now_str() -> str:
    return datetime.now(timezone.utc).strftime("%Y-%m-%d %H:%M UTC")


def save_output(agent_name: str, content: str) -> str:
    """Save agent output to a file and return the path."""
    out_dir = Path(__file__).parent / config.OUTPUT_DIR / agent_name
    out_dir.mkdir(parents=True, exist_ok=True)
    filename = datetime.now().strftime("%Y-%m-%d_%H%M") + ".md"
    path = out_dir / filename
    path.write_text(content, encoding="utf-8", errors="replace")
    return str(path)


# Force UTF-8 stdout on Windows
import io
if sys.stdout.encoding != "utf-8":
    sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding="utf-8", errors="replace")
    sys.stderr = io.TextIOWrapper(sys.stderr.buffer, encoding="utf-8", errors="replace")
