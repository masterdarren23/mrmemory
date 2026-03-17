"""Integration test using the Python SDK against live API.

Tests the SDK's actual interface (remember/recall/forget/memories)
by adapting it to our v1 API endpoints.
"""
import sys
import os
import time
import json
import urllib.request

BASE = "https://amr-memory-api.fly.dev"
PASS = 0
FAIL = 0

def test(name, condition):
    global PASS, FAIL
    if condition:
        PASS += 1
        print(f"  OK {name}")
    else:
        FAIL += 1
        print(f"  FAIL {name}")

def api(method, path, body=None, key=None):
    url = f"{BASE}{path}"
    data = json.dumps(body).encode() if body else None
    r = urllib.request.Request(url, data=data, method=method)
    r.add_header("Content-Type", "application/json")
    if key:
        r.add_header("Authorization", f"Bearer {key}")
    try:
        resp = urllib.request.urlopen(r)
        return resp.status, json.loads(resp.read()) if resp.status != 204 else None
    except urllib.error.HTTPError as e:
        return e.code, None

# Get API key
_, kd = api("POST", "/v1/auth/keys", {"name": "sdk-integration"})
KEY = kd["key"]
print(f"API Key: {kd['id']}")

# Now test the SDK
sys.path.insert(0, r"C:\Users\johnl\.openclaw\workspace-sdk\python-sdk\src")
from amr import AMR

# The SDK uses different endpoint paths than our API.
# Let's test it by pointing it at our API and see what happens.
# First, let's see if the SDK can be configured to work.

print("\n=== SDK Client Init ===")
try:
    client = AMR(
        api_key=KEY,
        base_url=f"{BASE}/v1",  # SDK prepends paths to this
        agent_id="sdk-test",
        namespace="sdk-test",
        timeout=30.0,
    )
    test("client created", True)
except Exception as e:
    test(f"client created: {e}", False)
    sys.exit(1)

# The SDK calls POST /remember but our API has POST /v1/memories
# The SDK calls POST /recall but our API has GET /v1/memories/recall
# We need to check if the SDK can work or needs adaptation

print("\n=== SDK remember() ===")
try:
    mem = client.remember("Integration test: user likes pizza", tags=["food", "preferences"])
    test("remember() succeeded", True)
    test(f"got memory id: {mem.id}", mem.id.startswith("mem_"))
    test("content preserved", mem.content == "Integration test: user likes pizza")
    test("tags preserved", mem.tags == ["food", "preferences"])
    MEM_ID = mem.id
except Exception as e:
    test(f"remember() failed: {e}", False)
    MEM_ID = None

print("\n=== SDK memories() (list) ===")
try:
    mems = client.memories()
    test("memories() succeeded", True)
    test("returns list", isinstance(mems, list))
    if mems:
        test("first item has id", hasattr(mems[0], 'id'))
except Exception as e:
    test(f"memories() failed: {e}", False)

print("\n=== SDK recall() ===")
time.sleep(2)
try:
    results = client.recall("what food does user enjoy", limit=5, threshold=0.3)
    test("recall() succeeded", True)
    test("returns list", isinstance(results, list))
    if results:
        test("found pizza memory", any("pizza" in m.content for m in results))
        test("has score", results[0].score is not None)
    else:
        test("found results (empty)", False)
except Exception as e:
    test(f"recall() failed: {e}", False)

print("\n=== SDK forget() ===")
if MEM_ID:
    try:
        client.forget(MEM_ID)
        test("forget() succeeded", True)
    except Exception as e:
        test(f"forget() failed: {e}", False)

client.close()

print(f"\n{'='*50}")
print(f"  PASSED: {PASS}")
print(f"  FAILED: {FAIL}")
print(f"  TOTAL:  {PASS + FAIL}")
print(f"{'='*50}")
sys.exit(0 if FAIL == 0 else 1)
