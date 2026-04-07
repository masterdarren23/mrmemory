-- Add memory_type to memories table
ALTER TABLE memories ADD COLUMN memory_type TEXT NOT NULL DEFAULT 'core' CHECK (memory_type IN ('core', 'private', 'provisional'));
CREATE INDEX idx_memories_type ON memories (tenant_id, memory_type);

-- Proposals table for provisional → core promotion
CREATE TABLE memory_proposals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    external_id TEXT NOT NULL UNIQUE,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    memory_id UUID NOT NULL REFERENCES memories(id) ON DELETE CASCADE,
    proposed_by TEXT NOT NULL,
    justification TEXT NOT NULL,
    evidence_ids TEXT[] NOT NULL DEFAULT '{}',
    confidence REAL DEFAULT 0.8,
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'approved', 'rejected', 'merged')),
    judge_model TEXT,
    judge_decision JSONB,
    decided_at TIMESTAMPTZ,
    decided_by TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at TIMESTAMPTZ
);
CREATE INDEX idx_proposals_tenant_status ON memory_proposals (tenant_id, status);
CREATE INDEX idx_proposals_memory ON memory_proposals (memory_id);

-- Add judge_model to tenants for per-tenant judge config
ALTER TABLE tenants ADD COLUMN judge_model TEXT DEFAULT 'gpt-4o-mini';
-- Add provisional_ttl_days to tenants
ALTER TABLE tenants ADD COLUMN provisional_ttl_days INTEGER DEFAULT 7;
