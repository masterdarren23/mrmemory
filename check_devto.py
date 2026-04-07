import requests
r = requests.get('https://dev.to/api/articles?username=realmrmemory', headers={'api-key': 'uRut7ESDweUDGwXd5TPsQWPM'})
for a in r.json():
    print(f"{a['published_at'][:10]} | {a['title'][:70]} | {a['url']}")
