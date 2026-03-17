import urllib.request, json, ssl

ctx = ssl.create_default_context()
key = json.load(open(r'C:\Users\johnl\.config\simmer\credentials.json'))['api_key']

# Try a market we haven't touched: NYC 33F or below NO
data = json.dumps({
    'market_id': '18f56021-306b-4e8a-8c38-89f15ad632ba',
    'side': 'no',
    'amount': 10,
    'venue': 'simmer',
    'source': 'sdk:weather',
    'reasoning': 'NOAA forecasts 37F for NYC Feb 13, above 33F threshold'
}).encode()
req = urllib.request.Request('https://api.simmer.markets/api/sdk/trade', data=data, headers={'Authorization': f'Bearer {key}', 'Content-Type': 'application/json'}, method='POST')
print("Placing trade (NYC 33- NO)...")
try:
    resp = urllib.request.urlopen(req, timeout=60, context=ctx)
    print(f"Response: {resp.read().decode()}")
except Exception as e:
    print(f"Error: {e}")
    if hasattr(e, 'read'):
        print(f"Body: {e.read().decode()}")
