# Stripe Setup Instructions

## Payment Link Configuration

Payment link: https://buy.stripe.com/9B6eVf0Jw0GefiP36l8g000

### After Payment Redirect

In the Stripe Dashboard, configure the payment link's **"After payment"** redirect URL to:

```
https://amr-memory-api.fly.dev/v1/welcome?session_id={CHECKOUT_SESSION_ID}
```

Steps:
1. Go to https://dashboard.stripe.com/payment-links
2. Click on the AMR payment link
3. Click "Edit"
4. Under "After payment" → select "Don't show confirmation page"
5. Set redirect URL to: `https://amr-memory-api.fly.dev/v1/welcome?session_id={CHECKOUT_SESSION_ID}`
6. Save

## Webhook Configuration

1. Go to https://dashboard.stripe.com/webhooks
2. Click "Add endpoint"
3. URL: `https://amr-memory-api.fly.dev/v1/billing/webhook`
4. Events to listen for: `checkout.session.completed`
5. Copy the **Signing secret** (starts with `whsec_`)

## Fly.io Secrets

Set these secrets on the Fly.io app:

```bash
fly secrets set STRIPE_WEBHOOK_SECRET=whsec_... STRIPE_SECRET_KEY=sk_live_...
```
