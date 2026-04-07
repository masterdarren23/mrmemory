# AMR Team Setup Instructions

Step-by-step guide to get all 6 agents running in OpenClaw.

---

## Prerequisites

- OpenClaw installed and gateway running
- Mr. Dogood (main agent) already configured
- Anthropic API key with Sonnet access

---

## Step 1: Add the 5 New Agents

Run each command:

```bash
openclaw agents add --id backend --name "Backend" --model "anthropic/claude-sonnet-4-20250514"
openclaw agents add --id sdk --name "SDK" --model "anthropic/claude-sonnet-4-20250514"
openclaw agents add --id frontend --name "Frontend" --model "anthropic/claude-sonnet-4-20250514"
openclaw agents add --id marketing --name "Marketing" --model "anthropic/claude-sonnet-4-20250514"
openclaw agents add --id devops --name "DevOps" --model "anthropic/claude-sonnet-4-20250514"
```

## Step 2: Copy Workspace Files

Each agent needs its SOUL.md, AGENTS.md, and USER.md in its workspace directory.

The files are in `amr-project/agent-{name}/`. Copy them to each agent's workspace:

```powershell
# Find your OpenClaw workspace root (typically ~/.openclaw/workspace)
$ws = "$HOME\.openclaw\workspace"

# Backend
Copy-Item "$ws\amr-project\agent-backend\*" "$ws\agent-backend\" -Force

# SDK
Copy-Item "$ws\amr-project\agent-sdk\*" "$ws\agent-sdk\" -Force

# Frontend
Copy-Item "$ws\amr-project\agent-frontend\*" "$ws\agent-frontend\" -Force

# Marketing
Copy-Item "$ws\amr-project\agent-marketing\*" "$ws\agent-marketing\" -Force

# DevOps
Copy-Item "$ws\amr-project\agent-devops\*" "$ws\agent-devops\" -Force
```

> **Note:** If your agents use different workspace paths, adjust accordingly. Check your `openclaw.json` for the actual workspace config per agent.

## Step 3: Update openclaw.json

Open your OpenClaw config:

```powershell
notepad "$HOME\.openclaw\openclaw.json"
```

Reference `amr-project/openclaw-config-snippet.json5` and add:

1. The 5 agent entries to `agents.list`
2. The `agents.bindings` section for inter-agent communication

Key communication paths:
- **main** ↔ all agents (hub)
- **backend** ↔ sdk, devops (technical coordination)
- **sdk** ↔ frontend (code examples for docs)
- **frontend** ↔ marketing (copy and messaging)

## Step 4: Copy Shared Project Files

Make sure all agents can access the architecture and plan:

```powershell
# The AMR-ARCHITECTURE.md and PROJECT-PLAN.md are in amr-project/
# Reference them from agent workspace files (already configured in AGENTS.md)
```

## Step 5: Restart Gateway

```bash
openclaw gateway restart
```

## Step 6: Verify All Agents

```bash
openclaw gateway status
```

You should see 6 agents listed: main, backend, sdk, frontend, marketing, devops.

Test each agent with a quick message:

```bash
openclaw chat backend "What's your role? Read your SOUL.md and introduce yourself."
openclaw chat sdk "What's your role? Read your SOUL.md and introduce yourself."
openclaw chat frontend "What's your role? Read your SOUL.md and introduce yourself."
openclaw chat marketing "What's your role? Read your SOUL.md and introduce yourself."
openclaw chat devops "What's your role? Read your SOUL.md and introduce yourself."
```

Each should respond in character with their personality and role.

---

## Quick Troubleshooting

| Issue | Fix |
|-------|-----|
| Agent not found | Check `agents.list` in openclaw.json has the agent id |
| Agent can't read files | Verify workspace path and file permissions |
| Inter-agent messages fail | Check `agents.bindings` config |
| Gateway won't start | Run `openclaw gateway status` for error details |

---

## What's Next

Once all agents are running:

1. Have Mr. Dogood kick off Week 1 by messaging each agent with their deliverables
2. Agents can message each other through the configured bindings
3. Check `PROJECT-PLAN.md` for the 8-week roadmap
4. Read `AMR-ARCHITECTURE.md` for the technical spec

Ship it. 🚀
