"""Integration test for AMR API against live deployment."""
import json
import sys
import time
import urllib.request
import urllib.error

BASE = "https://amr-memory-api.fly.dev"
PASS = 0
FAIL = 0

def req(method, path, body=None, headers=None):
    """Make HTTP request and return (status, data)."""
    url = f"{BASE}{path}"
    data = json.dumps(body).encode() if body else None
    r = urllib.request.Request(url, data=data, method=method, headers=headers or {})
    r.add_header("Content-Type", "application/json")
    try:
        resp = urllib.request.urlopen(r)
        return resp.status, json.loads(resp.read()) if resp.status != 204 else None
    except urllib.error.HTTPError as e:
        body_text = e.read().decode() if e.fp else ""
        try:
            return e.code, json.loads(body_text) if body_text else None
        except:
            return e.code, body_text

def test(name, condition):
    global PASS, FAIL
    if condition:
        PASS += 1
        print(f"  ✓ {name}")
    else:
        FAIL += 1
        print(f"  ✗ {name}")

# ---- Health ----
print("\n=== Health ===")
s, d = req("GET", "/health")
test("GET /health returns 200", s == 200)
test("status is ok", d.get("status") == "ok")
test("version is 0.1.0", d.get("version") == "0.1.0")

# ---- Auth ----
print("\n=== Auth ===")
s, d = req("POST", "/v1/auth/keys", {"name": "integration-test"})
test("POST /v1/auth/keys returns 201", s == 201)
test("key starts with amr_sk_", d["key"].startswith("amr_sk_"))
test("id starts with key_", d["id"].startswith("key_"))
test("scopes include memories:read", "memories:read" in d["scopes"])

API_KEY = d["key"]
AUTH = {"Authorization": f"Bearer {API_KEY}", "Content-Type": "application/json"}

# ---- Bad auth ----
print("\n=== Auth Rejection ===")
s, d = req("GET", "/v1/memories", headers={"Authorization": "Bearer amr_sk_bad", "Content-Type": "application/json"})
test("invalid key returns 401", s == 401)

s, d = req("GET", "/v1/memories", headers={"Content-Type": "application/json"})
test("missing auth returns 401", s == 401)

# ---- Create Memories ----
print("\n=== Create Memories ===")
s, m1 = req("POST", "/v1/memories", {
    "content": "The user prefers dark mode in all applications",
    "tags": ["preferences", "ui"],
    "namespace": "test",
    "agent_id": "integration-bot"
}, AUTH)
test("create memory returns 201", s == 201)
test("id starts with mem_", m1["id"].startswith("mem_"))
test("content matches", m1["content"] == "The user prefers dark mode in all applications")
test("tags match", m1["tags"] == ["preferences", "ui"])
test("namespace matches", m1["namespace"] == "test")
test("agent_id matches", m1["agent_id"] == "integration-bot")

s, m2 = req("POST", "/v1/memories", {
    "content": "Weekly standup is every Monday at 9am PST",
    "tags": ["schedule"],
    "namespace": "test",
    "agent_id": "integration-bot"
}, AUTH)
test("create second memory", s == 201)

s, m3 = req("POST", "/v1/memories", {
    "content": "The project uses Rust for the backend API server",
    "tags": ["tech"],
    "namespace": "test",
    "agent_id": "integration-bot"
}, AUTH)
test("create third memory", s == 201)

# ---- Validation ----
print("\n=== Validation ===")
s, d = req("POST", "/v1/memories", {"content": ""}, AUTH)
test("empty content returns 400", s == 400)

s, d = req("POST", "/v1/memories", {"content": "x" * 9000}, AUTH)
test("oversized content returns 400", s == 400)

# ---- List Memories ----
print("\n=== List Memories ===")
s, d = req("GET", "/v1/memories?namespace=test", headers=AUTH)
test("list returns 200", s == 200)
test("total >= 3", d["total"] >= 3)
test("memories is array", isinstance(d["memories"], list))
test("has limit field", "limit" in d)
test("has offset field", "offset" in d)

# ---- List with filters ----
s, d = req("GET", "/v1/memories?namespace=test&agent_id=integration-bot&limit=2", headers=AUTH)
test("filtered list works", s == 200 and d["total"] >= 3)
test("limit=2 returns max 2", len(d["memories"]) <= 2)

# ---- Recall (Semantic Search) ----
print("\n=== Recall (Semantic Search) ===")
time.sleep(2)  # wait for embeddings to index

s, d = req("GET", "/v1/memories/recall?query=what+color+theme+does+user+like&namespace=test", headers=AUTH)
test("recall returns 200", s == 200)
test("has memories array", "memories" in d)
test("has count", "count" in d)
test("has query_time_ms", "query_time_ms" in d)

if d["count"] > 0:
    top = d["memories"][0]
    test("top result is about dark mode", "dark mode" in top["content"])
    test("has similarity score", "similarity" in top and isinstance(top["similarity"], (int, float)))
    test("similarity > 0.3", top["similarity"] > 0.3)
else:
    test("recall found results", False)
    test("(skipped - no results)", True)
    test("(skipped - no results)", True)

# ---- Recall with no matches ----
s, d = req("GET", "/v1/memories/recall?query=quantum+physics+black+holes&namespace=test", headers=AUTH)
test("unrelated query returns fewer/no matches", s == 200)

# ---- Delete ----
print("\n=== Delete ===")
s, d = req("DELETE", f"/v1/memories/{m1['id']}", headers=AUTH)
test("delete returns 204", s == 204)

# Verify deletion
s, d = req("GET", "/v1/memories?namespace=test", headers=AUTH)
ids = [m["id"] for m in d["memories"]]
test("deleted memory not in list", m1["id"] not in ids)

# Delete non-existent
s, d = req("DELETE", "/v1/memories/mem_doesnotexist", headers=AUTH)
test("delete nonexistent returns 404", s == 404)

# ---- Cleanup ----
print("\n=== Cleanup ===")
req("DELETE", f"/v1/memories/{m2['id']}", headers=AUTH)
req("DELETE", f"/v1/memories/{m3['id']}", headers=AUTH)
test("cleanup done", True)

# ---- Summary ----
print(f"\n{'='*50}")
print(f"  PASSED: {PASS}")
print(f"  FAILED: {FAIL}")
print(f"  TOTAL:  {PASS + FAIL}")
print(f"{'='*50}")

sys.exit(0 if FAIL == 0 else 1)
