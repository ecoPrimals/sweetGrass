-- Initial migration for SweetGrass PostgreSQL store
-- Creates the braids table for storing provenance data

CREATE TABLE IF NOT EXISTS braids (
    id TEXT PRIMARY KEY,
    data_hash TEXT NOT NULL,
    content JSONB NOT NULL,
    mime_type TEXT,
    size_bytes BIGINT,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Create indexes for common queries
CREATE INDEX IF NOT EXISTS idx_braids_data_hash ON braids(data_hash);
CREATE INDEX IF NOT EXISTS idx_braids_created_at ON braids(created_at);
CREATE INDEX IF NOT EXISTS idx_braids_content_gin ON braids USING GIN (content);

-- Enable UUID extension if needed
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

