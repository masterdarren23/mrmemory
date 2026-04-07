ALTER TABLE memories ADD COLUMN created_by TEXT NOT NULL DEFAULT 'unknown';
ALTER TABLE memories ADD COLUMN last_modified_by TEXT;
ALTER TABLE memories ADD COLUMN confidence REAL DEFAULT 1.0;
ALTER TABLE memories ADD COLUMN write_source TEXT DEFAULT 'api';
CREATE INDEX idx_memories_created_by ON memories (tenant_id, created_by);
