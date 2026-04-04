"""Run a single marketing agent by name. Used by Task Scheduler."""

import sys
import os

# Set working directory to script location
os.chdir(os.path.dirname(os.path.abspath(__file__)))

import config
config.load_env()

AGENTS = {
    "growth_metrics": "agents.growth_metrics",
    "social_listening": "agents.social_listening",
    "social_media": "agents.social_media",
    "media_outreach": "agents.media_outreach",
    "seo": "agents.seo",
    "geo": "agents.geo",
    "cvr": "agents.cvr",
    "competitor_watch": "agents.competitor_watch",
    "community_reply": "agents.community_reply",
    "weekly_strategy": "agents.weekly_strategy",
}


def main():
    if len(sys.argv) < 2:
        print("Usage: python run_agent.py <agent_name>")
        print(f"Available agents: {', '.join(AGENTS.keys())}")
        print("\nOr use 'all' to run all agents.")
        sys.exit(1)

    agent_name = sys.argv[1].lower()

    if agent_name == "all":
        for name in AGENTS:
            print(f"\n{'='*60}")
            print(f"Running: {name}")
            print('='*60)
            _run_agent(name)
        return

    if agent_name not in AGENTS:
        print(f"Unknown agent: {agent_name}")
        print(f"Available: {', '.join(AGENTS.keys())}")
        sys.exit(1)

    _run_agent(agent_name)


def _run_agent(name: str):
    import importlib
    try:
        module = importlib.import_module(AGENTS[name])
        module.run()
    except Exception as e:
        print(f"[ERROR] Agent {name} failed: {e}")
        from core import log_run
        log_run(name, str(e), status="error")


if __name__ == "__main__":
    main()
