import hashlib, secrets, uuid

BASE62 = '0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz'
raw_bytes = secrets.token_bytes(32)
encoded = ''
for b in raw_bytes:
    encoded += BASE62[(b // 62) % 62]
    encoded += BASE62[b % 62]
key = f'amr_sk_{encoded}'

key_hash = hashlib.sha256(key.encode()).hexdigest()
prefix = encoded[:8]
ext_id = f'key_{uuid.uuid4().hex[:12]}'
tenant_id = '0e86e7de-0081-4257-b0bc-006efd0238cc'

print(f'KEY={key}')
print()
print(f"INSERT INTO api_keys (id, external_id, tenant_id, name, key_hash, key_prefix, scopes) VALUES (gen_random_uuid(), '{ext_id}', '{tenant_id}', 'fulltest-key', '\\x{key_hash}', '{prefix}', ARRAY['memories:read','memories:write','memories:delete','usage:read']);")
