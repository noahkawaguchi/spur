-- Add migration script here
CREATE TABLE prompts (
    id SERIAL PRIMARY KEY,
    author_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    body TEXT NOT NULL CHECK (trim(body) <> ''),
    body_hash TEXT GENERATED ALWAYS AS (md5(body)) STORED,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (author_id, body_hash)
);
