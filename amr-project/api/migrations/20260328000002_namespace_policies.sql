CREATE TABLE namespace_policies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    namespace TEXT NOT NULL,
    policy TEXT NOT NULL DEFAULT 'open' CHECK (policy IN ('open', 'append_only', 'read_only')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(tenant_id, namespace)
);
