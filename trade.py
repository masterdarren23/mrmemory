import urllib.request, json, ssl, time

ctx = ssl.create_default_context()
key = json.load(open(r'C:\Users\johnl\.config\simmer\credentials.json'))['api_key']

def trade(mid, side, amount, reasoning):
    data = json.dumps({'market_id': mid, 'side': side, 'amount': amount, 'venue': 'simmer', 'source': 'sdk:weather', 'reasoning': reasoning}).encode()
    req = urllib.request.Request('https://api.simmer.markets/api/sdk/trade', data=data, headers={'Authorization': f'Bearer {key}', 'Content-Type': 'application/json'}, method='POST')
    try:
        resp = urllib.request.urlopen(req, timeout=30, context=ctx)
        r = json.loads(resp.read())
        sb = r.get('shares_bought', 0)
        cost = r.get('cost', 0)
        bal = r.get('balance', '?')
        err = r.get('error', '')
        print(f"  OK: shares={sb} cost={cost} balance={bal} error={err}")
    except Exception as e:
        print(f"  ERR: {e}")
    time.sleep(2)

print("=== Seattle 52+ YES ===")
trade('ccef0502-1188-4a1f-8d16-c29c717705a9', 'yes', 20, 'NOAA forecasts 53F for Seattle Feb 12, above 52F threshold')

print("=== Chicago 39- NO ===")
trade('76196437-9d04-447b-b825-01e2b953e520', 'no', 15, 'NOAA forecasts 48F for Chicago Feb 13, well above 39F')

print("=== NYC 38-39 YES ===")
trade('af3141b6-dece-4e48-8fab-7752badaa7cf', 'yes', 20, 'NOAA forecasts 37F for NYC Feb 13, forecast uncertainty makes 38-39F plausible')

print("=== Atlanta 60-61 YES ===")
trade('8188e6e7-9376-466e-90f7-a4fe678e463c', 'yes', 15, 'NOAA forecasts 61F for Atlanta Feb 12, direct hit on 60-61F bucket')

print("=== Atlanta 56-57 NO ===")
trade('7240ed18-56a5-4ba1-9a8c-c6c6f2b7da06', 'no', 15, 'NOAA forecasts 62F for Atlanta Feb 13, above 56-57F')

print("=== Dallas 66-67 NO ===")
trade('7302e14d-f9bf-4042-9cb7-e59b5fd63d48', 'no', 15, 'NOAA forecasts 79F for Dallas Feb 13, way above 66-67F')
