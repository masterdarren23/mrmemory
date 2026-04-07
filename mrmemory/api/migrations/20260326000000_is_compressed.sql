-- Add is_compressed flag and merged_from tracking
ALTER TABLE memories ADD COLUMN is_compressed BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE memories ADD COLUMN merged_from TEXT[] NOT NULL DEFAULT '{}';
