-- Previously applied Stripe billing migration (placeholder for compatibility)
-- Actual Stripe fields are in 20260317000000_stripe_fields.sql
ALTER TABLE tenants ADD COLUMN IF NOT EXISTS stripe_customer_id TEXT;
ALTER TABLE tenants ADD COLUMN IF NOT EXISTS stripe_subscription_id TEXT;
ALTER TABLE tenants ADD COLUMN IF NOT EXISTS subscription_status TEXT;
