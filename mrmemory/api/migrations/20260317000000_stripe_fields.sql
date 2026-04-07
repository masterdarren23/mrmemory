-- Add Stripe fields to tenants (idempotent)
ALTER TABLE tenants ADD COLUMN IF NOT EXISTS stripe_customer_id TEXT;
ALTER TABLE tenants ADD COLUMN IF NOT EXISTS stripe_session_id TEXT;

CREATE INDEX IF NOT EXISTS idx_tenants_stripe_customer_id ON tenants (stripe_customer_id);
CREATE INDEX IF NOT EXISTS idx_tenants_stripe_session_id ON tenants (stripe_session_id);
