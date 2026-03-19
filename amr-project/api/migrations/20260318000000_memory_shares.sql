-- Memory sharing ACLs for WebSocket real-time push (idempotent)
CREATE TABLE IF NOT EXISTS memory_shares (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id   UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    agent_id    TEXT NOT NULL,           -- the agent granting access
    target_agent_id TEXT NOT NULL,       -- the agent receiving access
    namespace   TEXT NOT NULL DEFAULT '*', -- '*' means all namespaces
    permission  TEXT NOT NULL DEFAULT 'read' CHECK (permission IN ('read', 'readwrite')),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (tenant_id, agent_id, target_agent_id, namespace)
);

CREATE INDEX IF NOT EXISTS idx_memory_shares_target ON memory_shares (tenant_id, target_agent_id);
CREATE INDEX IF NOT EXISTS idx_memory_shares_agent  ON memory_shares (tenant_id, agent_id);
