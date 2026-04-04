"""Marketing Engine Configuration"""

# Ollama
OLLAMA_URL = "http://localhost:11434"
OLLAMA_MODEL = "llama3:latest"

# Brave Search API (free tier: 2,000 queries/mo)
BRAVE_API_KEY = ""  # Will pull from env

# Telegram delivery
TELEGRAM_BOT_TOKEN = ""  # Will pull from env
TELEGRAM_CHAT_ID = "8583218792"  # Dirwood

# Product context injected into every agent prompt
PRODUCT_CONTEXT = """
PRODUCT: MrMemory — Managed Memory API for AI Agents
WEBSITE: mrmemory.dev
DOCS: mrmemory.dev/docs
GITHUB: github.com/masterdarren23/mrmemory
INSTALL: pip install mrmemory / npm install memorymr
PRICE: $5/mo with 7-day free trial
STRIPE: buy.stripe.com/00w4gB2REex4daHeP38g001
STACK: Rust/Axum backend, PostgreSQL, Qdrant vector DB, OpenAI embeddings
KEY FEATURES: Semantic recall, auto-remember, memory compression (40-60% token savings), LangChain integration, self-edit tools, three-layer governance, anti-pollution
COMPETITOR: Mem0 (mem0.ai, $24M funded) — lacks compression, self-edit, governance
OTHER COMPETITORS: Zep (self-host only), Letta/MemGPT (self-host only)
TRACTION: 146 unique GitHub cloners in 6 hours from one Reddit post, $0 ad spend
TARGET AUDIENCE: AI developers, startups using LangChain/CrewAI/AutoGen/OpenAI Agents
"""

# Database
DB_PATH = "data/marketing.db"

# Output directory
OUTPUT_DIR = "outputs"

import os
from pathlib import Path

def load_env():
    """Load config from .env file and environment variables."""
    global BRAVE_API_KEY, TELEGRAM_BOT_TOKEN
    
    # Load .env file if it exists
    env_path = Path(__file__).parent / ".env"
    if env_path.exists():
        for line in env_path.read_text().strip().splitlines():
            line = line.strip()
            if line and not line.startswith("#") and "=" in line:
                key, val = line.split("=", 1)
                os.environ.setdefault(key.strip(), val.strip())
    
    BRAVE_API_KEY = os.environ.get("BRAVE_API_KEY", BRAVE_API_KEY)
    TELEGRAM_BOT_TOKEN = os.environ.get("MKT_TELEGRAM_TOKEN", TELEGRAM_BOT_TOKEN)
