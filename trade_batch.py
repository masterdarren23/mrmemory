import urllib.request, json, ssl, time

ctx = ssl.create_default_context()
key = json.load(open(r'C:\Users\johnl\.config\simmer\credentials.json'))['api_key']

trades = [
    ('ccef0502-1188-4a1f-8d16-c29c717705a9', 'yes', 20, 'NOAA forecasts 53F for Seattle Feb 12, above 52F threshold'),
    ('76196437-9d04-447b-b825-01e2b953e520', 'no', 15, 'NOAA forecasts 48F for Chicago Feb 13, well above 39F'),
    ('af3141b6-dece-4e48-8fab-7752badaa7cf', 'yes', 20, 'NOAA forecasts 37F for NYC Feb 13, forecast uncertainty makes 38-39F plausible'),
    ('8188e6e7-9376-466e-90f7-a4fe678e463c', 'yes', 15, 'NOAA forecasts 61F for Atlanta Feb 12, direct hit on 60-61F bucket'),
    ('7240ed18-56a5-4ba1-9a8c-c6c6f2b7da06', 'no', 15, 'NOAA forecasts 62F for Atlanta Feb 13, above 56-57F'),
    ('7302e14d-f9bf-4042-9cb7-e59b5fd63d48', 'no', 15, 'NOAA forecasts 79F for Dallas Feb 13, way above 66-67F'),
]

for mid, side, amount, reasoning in trades:
    data = json.dumps({'market_id': mid, 'side': side, 'amount': amount, 'venue': 'simmer', 'source': 'sdk:weather', 'reasoning': reasoning}).encode()
    req = urllib.request.Request('https://api.simmer.markets/api/sdk/trade', data=data, headers={'Authorization': f'Bearer {key}', 'Content-Type': 'application/json'}, method='POST')
    label = reasoning[:50]
    print(f"Trading: {label}...")
    try:
        resp = urllib.request.urlopen(req, timeout=90, context=ctx)
        r = json.loads(resp.read().decode())
        if r['success']:
            print(f"  OK: {r['shares_bought']:.2f} shares, cost ${r['cost']:.2f}, balance ${r['balance']:.2f}")
        else:
            print(f"  FAIL: {r.get('error','unknown')}")
    except Exception as e:
        print(f"  ERR: {e}")
    time.sleep(1)

print("\nDone!")
