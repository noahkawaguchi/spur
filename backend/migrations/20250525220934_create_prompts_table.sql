CREATE TABLE prompts (
    id INT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    author_id INT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    body TEXT NOT NULL CHECK (trim(body) <> ''),
    body_hash TEXT GENERATED ALWAYS AS (md5(body)) STORED,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (author_id, body_hash)
);
