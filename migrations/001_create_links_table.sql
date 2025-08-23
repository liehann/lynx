-- Create links table
CREATE TABLE links (
    id SERIAL PRIMARY KEY,
    host TEXT NOT NULL,
    source TEXT NOT NULL,
    target TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT now(),
    
    -- Ensure uniqueness of (host, source) combination
    UNIQUE(host, source)
);

-- Index for faster lookups
CREATE UNIQUE INDEX idx_links_host_source ON links (host, source);
CREATE INDEX idx_links_created_at ON links(created_at DESC);
