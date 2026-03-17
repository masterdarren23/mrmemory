import urllib.request, json, ssl

ctx = ssl.create_default_context()
key = json.load(open(r'C:\Users\johnl\.config\simmer\credentials.json'))['api_key']

# Try dry run first
data = json.dumps({'market_id': 'ccef0502-1188-4a1f-8d16-c29c717705a9', 'side': 'yes', 'amount': 5, 'venue': 'simmer', 'dry_run': True}).encode()
req = urllib.request.Request('https://api.simmer.markets/api/sdk/trade', data=data, headers={'Authorization': f'Bearer {key}', 'Content-Type': 'application/json'}, method='POST')
print("Sending dry run...")
try:
    resp = urllib.request.urlopen(req, timeout=15, context=ctx)
    print(f"Response: {resp.read().decode()}")
except Exception as e:
    print(f"Error: {e}")
    if hasattr(e, 'read'):
        print(f"Body: {e.read().decode()}")
