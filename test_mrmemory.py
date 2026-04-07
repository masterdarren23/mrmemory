"""Full integration test of MrMemory API."""
import requests, json, time, sys

BASE = 'https://amr-memory-api.fly.dev'
API_KEY = 'amr_sk_0j1e2o1a183s0K400N3V0f1J213M1l0F2Q2j1Y2b2x3v0a3e1x1s2V2H3e301A1S'
H = {"Authorization": f"Bearer {API_KEY}", "Content-Type": "application/json"}
PASS_COUNT = 0
FAIL_COUNT = 0
FAILURES = []

def test(name, response, expect_status=None):
    global PASS_COUNT, FAIL_COUNT
    status = response.status_code
    if expect_status:
        ok = status == expect_status
    else:
        ok = 200 <= status < 300
    if ok:
        PASS_COUNT += 1
        print(f"  PASS [{status}] {name}")
    else:
        FAIL_COUNT += 1
        FAILURES.append((name, status, response.text[:200]))
        print(f"  FAIL [{status}] {name}")
        print(f"    {response.text[:200]}")
    return response

print("=" * 60)
print("MrMemory Full API Test (post-deploy)")
print("=" * 60)

# ---- Health ----
print("\n--- Health ---")
test("GET /health", requests.get(f"{BASE}/health"))
test("GET /health/ready", requests.get(f"{BASE}/health/ready"))

# ---- Auth ----
print("\n--- Auth ---")
test("Auth reject (bad key)", requests.get(f"{BASE}/v1/memories", headers={"Authorization": "Bearer amr_sk_bad"}), expect_status=401)
test("Auth reject (no header)", requests.get(f"{BASE}/v1/memories"), expect_status=401)

# ---- Create Memories ----
print("\n--- Create Memories ---")
mem_ids = []

r = test("POST /v1/memories (basic)", requests.post(f"{BASE}/v1/memories", headers=H, json={"content": "The user likes Python programming"}))
if r.status_code < 300:
    mem_ids.append(r.json()["id"])

r = test("POST /v1/memories (with tags)", requests.post(f"{BASE}/v1/memories", headers=H, json={"content": "The user prefers dark mode interfaces", "tags": ["preferences", "ui"]}))
if r.status_code < 300:
    mem_ids.append(r.json()["id"])

r = test("POST /v1/memories (with namespace)", requests.post(f"{BASE}/v1/memories", headers=H, json={"content": "Meeting with Alice on Friday at 3pm", "namespace": "calendar"}))
if r.status_code < 300:
    mem_ids.append(r.json()["id"])

r = test("POST /v1/memories (with agent_id)", requests.post(f"{BASE}/v1/memories", headers=H, json={"content": "User email is test@example.com", "agent_id": "assistant-v1"}))
if r.status_code < 300:
    mem_ids.append(r.json()["id"])

r = test("POST /v1/memories (with metadata)", requests.post(f"{BASE}/v1/memories", headers=H, json={"content": "Favorite color is blue", "metadata": {"source": "conversation", "confidence": "high"}}))
if r.status_code < 300:
    mem_ids.append(r.json()["id"])

print(f"    Created {len(mem_ids)} memories")

# ---- Validation ----
print("\n--- Validation ---")
test("Reject empty content", requests.post(f"{BASE}/v1/memories", headers=H, json={"content": ""}), expect_status=400)
test("Reject >20 tags", requests.post(f"{BASE}/v1/memories", headers=H, json={"content": "x", "tags": [f"t{i}" for i in range(25)]}), expect_status=400)

# ---- List Memories ----
print("\n--- List Memories ---")
r = test("GET /v1/memories", requests.get(f"{BASE}/v1/memories", headers=H))
if r.status_code < 300:
    print(f"    Total: {r.json().get('total', '?')}")

test("GET /v1/memories?namespace=calendar", requests.get(f"{BASE}/v1/memories?namespace=calendar", headers=H))

# ---- Get Single Memory ----
print("\n--- Get Single Memory ---")
if mem_ids:
    r = test("GET /v1/memories/:id", requests.get(f"{BASE}/v1/memories/{mem_ids[0]}", headers=H))
    if r.status_code < 300:
        print(f"    Content: {r.json().get('content', '')[:80]}")

# ---- Semantic Recall ----
print("\n--- Semantic Recall ---")
time.sleep(2)
r = test("Recall: programming language", requests.get(f"{BASE}/v1/memories/recall?query=programming+language&top_k=3", headers=H))
if r.status_code < 300:
    results = r.json()
    if isinstance(results, list):
        print(f"    {len(results)} results returned")
        for m in results[:2]:
            score = m.get('score', 'N/A')
            print(f"      - [{score}] {m.get('content', '')[:60]}")

test("Recall: UI preferences", requests.get(f"{BASE}/v1/memories/recall?query=UI+theme&top_k=2", headers=H))
test("Recall: meetings", requests.get(f"{BASE}/v1/memories/recall?query=upcoming+meetings&top_k=2", headers=H))

# ---- Update Memory ----
print("\n--- Update Memory (PATCH) ---")
if mem_ids:
    test("PATCH /v1/memories/:id", requests.patch(f"{BASE}/v1/memories/{mem_ids[0]}", headers=H, json={"content": "The user loves Python AND Rust"}))
    r = test("Verify update", requests.get(f"{BASE}/v1/memories/{mem_ids[0]}", headers=H))
    if r.status_code < 300:
        c = r.json().get("content", "")
        ok = "Rust" in c
        print(f"    Content correct: {ok} -> {c[:80]}")

# ---- Stats ----
print("\n--- Stats ---")
r = test("GET /v1/stats", requests.get(f"{BASE}/v1/stats", headers=H))
if r.status_code < 300:
    s = r.json()
    print(f"    Total memories: {s.get('total_memories')}, Plan: {s.get('plan')}")

# ---- Private Memory (Governance) ----
print("\n--- Private Memory ---")
r = test("POST /v1/memories/private", requests.post(f"{BASE}/v1/memories/private", headers=H, json={"content": "Temporary scratchpad note", "agent_id": "test-agent"}))
private_id = r.json().get("id") if r.status_code < 300 else None

# ---- Propose Memory (Governance) ----
print("\n--- Propose Memory ---")
# evidence is an array of memory IDs
r = test("POST /v1/memories/propose", requests.post(f"{BASE}/v1/memories/propose", headers=H, json={"content": "The user is a senior developer", "justification": "They mentioned 10 years of experience", "evidence": [mem_ids[0]] if mem_ids else []}))
proposal_id = None
if r.status_code < 300:
    data = r.json()
    proposal_id = data.get("id") or data.get("proposal_id") or data.get("external_id")
    print(f"    Result: {json.dumps(data)[:200]}")

# ---- List Proposals ----
r = test("GET /v1/memories/proposals", requests.get(f"{BASE}/v1/memories/proposals", headers=H))

# ---- Decide Proposal ----
if proposal_id:
    test("POST /proposals/:id/decide", requests.post(f"{BASE}/v1/memories/proposals/{proposal_id}/decide", headers=H, json={"decision": "approve"}))

# ---- Merge Memories ----
print("\n--- Merge Memories ---")
if len(mem_ids) >= 2:
    test("POST /v1/memories/merge", requests.post(f"{BASE}/v1/memories/merge", headers=H, json={"memory_ids": [mem_ids[0], mem_ids[1]], "merged_content": "User likes Python, Rust, and dark mode"}))

# ---- Delete Memory ----
print("\n--- Delete Memory ---")
if len(mem_ids) >= 3:
    test("DELETE /v1/memories/:id", requests.delete(f"{BASE}/v1/memories/{mem_ids[2]}", headers=H))
    test("Verify delete (404)", requests.get(f"{BASE}/v1/memories/{mem_ids[2]}", headers=H), expect_status=404)

# ---- Delete Outdated ----
test("DELETE /v1/memories/outdated", requests.delete(f"{BASE}/v1/memories/outdated", headers=H, json={"older_than_seconds": 999999999}))

# ---- Auto-Remember ----
print("\n--- Auto-Remember ---")
r = test("POST /v1/memories/auto", requests.post(f"{BASE}/v1/memories/auto", headers=H, json={"messages": [{"role": "user", "content": "I just moved to San Francisco"}, {"role": "assistant", "content": "How are you liking SF?"}]}))
if r.status_code < 300:
    print(f"    {r.json()}")

# ---- Compress ----
print("\n--- Compress ---")
r = test("POST /v1/memories/compress", requests.post(f"{BASE}/v1/memories/compress", headers=H, json={}))
if r.status_code < 300:
    print(f"    Status: {r.json().get('status')}")

# ---- Namespace Policies ----
print("\n--- Namespace Policies ---")
test("PUT namespace policy", requests.put(f"{BASE}/v1/namespaces/test-ns/policy", headers=H, json={"policy": "append_only"}))
r = test("GET namespace policy", requests.get(f"{BASE}/v1/namespaces/test-ns/policy", headers=H))
if r.status_code < 300:
    print(f"    Policy: {r.json()}")

# ---- WebSocket ----
print("\n--- WebSocket ---")
test("WS endpoint exists", requests.get(f"{BASE}/v1/ws", headers=H), expect_status=400)

# ---- Auth: Create Key ----
print("\n--- Create API Key ---")
r = test("POST /v1/auth/keys", requests.post(f"{BASE}/v1/auth/keys", headers=H, json={"name": "test-sub-key"}))
if r.status_code < 300:
    print(f"    New key prefix: {r.json().get('prefix')}")

# ---- Cleanup ----
print("\n--- Cleanup ---")
for mid in mem_ids:
    try: requests.delete(f"{BASE}/v1/memories/{mid}", headers=H)
    except: pass
if private_id:
    try: requests.delete(f"{BASE}/v1/memories/{private_id}", headers=H)
    except: pass
print("    Done")

# ---- Summary ----
total = PASS_COUNT + FAIL_COUNT
print("\n" + "=" * 60)
print(f"RESULTS: {PASS_COUNT} passed, {FAIL_COUNT} failed out of {total} tests")
if FAILURES:
    print("\nFailed:")
    for name, status, body in FAILURES:
        print(f"  [{status}] {name}: {body}")
else:
    print("\nALL TESTS PASSED!")
print("=" * 60)
