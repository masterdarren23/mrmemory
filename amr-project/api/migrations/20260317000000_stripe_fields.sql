-- Add Stripe fields to tenants
ALTER TABLE tenants ADD COLUMN stripe_customer_id TEXT;
ALTER TABLE tenants ADD COLUMN stripe_session_id TEXT;

CREATE INDEX idx_tenants_stripe_customer_id ON tenants (stripe_customer_id);
CREATE INDEX idx_tenants_stripe_session_id ON tenants (stripe_session_id);
