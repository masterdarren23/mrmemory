-- AMR Initial Schema
-- Run with: sqlx migrate run

-- ============================================================
-- TENANTS
-- ============================================================
CREATE TABLE tenants (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    external_id     TEXT NOT NULL UNIQUE,
    name            TEXT NOT NULL,
    email           TEXT NOT NULL UNIQUE,
    plan            TEXT NOT NULL DEFAULT 'starter'
                    CHECK (plan IN ('starter', 'pro', 'enterprise')),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_tenants_email ON tenants (email);
CREATE INDEX idx_tenants_external_id ON tenants (external_id);

-- ============================================================
-- API KEYS
-- ============================================================
CREATE TABLE api_keys (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    external_id     TEXT NOT NULL UNIQUE,
    tenant_id       UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    name            TEXT NOT NULL,
    key_hash        BYTEA NOT NULL,
    key_prefix      TEXT NOT NULL,
    scopes          TEXT[] NOT NULL DEFAULT '{memories:read,memories:write,memories:share,memories:delete,usage:read}',
    expires_at      TIMESTAMPTZ,
    revoked_at      TIMESTAMPTZ,
    last_used_at    TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_api_keys_tenant ON api_keys (tenant_id);
CREATE INDEX idx_api_keys_hash ON api_keys (key_hash);
CREATE INDEX idx_api_keys_prefix ON api_keys (key_prefix);

-- ============================================================
-- MEMORIES
-- ============================================================
CREATE TABLE memories (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    external_id     TEXT NOT NULL UNIQUE,
    tenant_id       UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    agent_id        TEXT NOT NULL,
    namespace       TEXT NOT NULL DEFAULT 'default',
    content         TEXT NOT NULL,
    tags            TEXT[] NOT NULL DEFAULT '{}',
    metadata        JSONB NOT NULL DEFAULT '{}',
    embedding_model TEXT,
    vector_indexed  BOOLEAN NOT NULL DEFAULT FALSE,
    ttl_seconds     INTEGER,
    expires_at      TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Primary query path
CREATE INDEX idx_memories_tenant_agent_ns
    ON memories (tenant_id, agent_id, namespace);

-- Tag filtering
CREATE INDEX idx_memories_tags
    ON memories USING GIN (tags);

-- TTL cleanup
CREATE INDEX idx_memories_expires
    ON memories (expires_at)
    WHERE expires_at IS NOT NULL;

-- Listing with time ordering
CREATE INDEX idx_memories_tenant_created
    ON memories (tenant_id, created_at DESC);

-- ============================================================
-- MEMORY SHARES
-- ============================================================
CREATE TABLE memory_shares (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    external_id         TEXT NOT NULL UNIQUE,
    memory_id           UUID NOT NULL REFERENCES memories(id) ON DELETE CASCADE,
    tenant_id           UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    source_agent_id     TEXT NOT NULL,
    target_agent_id     TEXT NOT NULL,
    permissions         TEXT NOT NULL DEFAULT 'read'
                        CHECK (permissions IN ('read', 'read_write')),
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_shares_memory ON memory_shares (memory_id);
CREATE INDEX idx_shares_target ON memory_shares (tenant_id, target_agent_id);
CREATE UNIQUE INDEX idx_shares_unique ON memory_shares (memory_id, target_agent_id);

-- ============================================================
-- USAGE TRACKING
-- ============================================================
CREATE TABLE usage_daily (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id       UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    date            DATE NOT NULL,
    api_calls       BIGINT NOT NULL DEFAULT 0,
    memories_stored BIGINT NOT NULL DEFAULT 0,
    UNIQUE (tenant_id, date)
);

CREATE INDEX idx_usage_tenant_date ON usage_daily (tenant_id, date DESC);
