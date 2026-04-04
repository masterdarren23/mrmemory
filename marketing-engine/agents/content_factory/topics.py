"""Topic generation and management for the Content Factory."""

# Pre-loaded topic queues — the factory cycles through these
# and also discovers new topics from web research

SEO_BLOG_TOPICS = [
    # High-intent keywords
    "How to Add Persistent Memory to LangChain Agents",
    "AI Agent Memory: Complete Guide for Developers (2026)",
    "Mem0 vs MrMemory: Which AI Memory API Should You Use?",
    "How to Make Your AI Agent Remember Conversations",
    "Building AI Agents That Never Forget: A Developer Guide",
    "LangChain Memory Solutions Compared: MrMemory, Mem0, Zep, MemGPT",
    "How to Reduce AI Token Costs with Memory Compression",
    "Adding Long-Term Memory to CrewAI Agents: Step-by-Step",
    "AutoGen + Persistent Memory: Complete Integration Tutorial",
    "OpenAI Agents SDK: How to Add Memory in 5 Minutes",
    "Why Your AI Chatbot Forgets Everything (And How to Fix It)",
    "AI Agent Memory API: Build vs Buy Analysis",
    "Self-Healing AI Memory: How Agents Can Manage Their Own Knowledge",
    "Memory Governance for AI Agents: Preventing Data Pollution",
    "Semantic Search vs Keyword Search for AI Agent Memory",
    "How to Build a Multi-Agent System with Shared Memory",
    "AI Agent Memory Benchmarks: Speed, Accuracy, and Cost Compared",
    "The Developer's Guide to AI Memory Compression",
    "RAG vs Agent Memory: When to Use Each",
    "How to Add Memory to Your Python AI Agent in 3 Lines of Code",
    
    # Comparison pages
    "MrMemory vs Mem0: Feature Comparison 2026",
    "MrMemory vs Zep: Managed vs Self-Hosted Memory",
    "MrMemory vs MemGPT/Letta: Which Is Right for You?",
    "Top 5 AI Agent Memory Solutions in 2026",
    
    # Integration tutorials
    "MrMemory + LangChain: Complete Integration Guide",
    "MrMemory + CrewAI: Give Your Crew Persistent Memory",
    "MrMemory + AutoGen: Multi-Agent Memory Tutorial",
    "MrMemory + OpenAI Agents SDK: Quick Start",
    "MrMemory + FastAPI: Building a Memory-Powered API",
    "MrMemory + Streamlit: Build a Chatbot That Remembers",
    
    # Technical deep dives
    "How Semantic Vector Search Works for AI Memory",
    "Memory Compression Algorithms for AI Agents Explained",
    "Three-Layer Memory Governance: Core, Provisional, Private",
    "Anti-Pollution Patterns for AI Agent Memory",
    "Designing Memory Schemas for Multi-Agent Systems",
]

STACKOVERFLOW_QUERIES = [
    "how to persist memory langchain agent",
    "AI chatbot remember previous conversations python",
    "langchain conversation memory between sessions",
    "store AI agent memory database",
    "AI agent context window too small solution",
    "langchain memory persist restart",
    "crewai agent memory",
    "autogen agent remember",
    "openai assistant memory between threads",
]

GITHUB_ISSUE_QUERIES = [
    "memory persistence site:github.com/langchain-ai",
    "long term memory site:github.com/joaomdmoura/crewAI",
    "agent memory site:github.com/microsoft/autogen",
    "memory between sessions site:github.com issues",
]
