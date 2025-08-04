CREATE TABLE posts (
    id INT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    author_id INT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    prompt_id INT NOT NULL REFERENCES prompts(id) ON DELETE CASCADE,
    body TEXT NOT NULL CHECK (trim(body) <> ''),
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    edited_at TIMESTAMPTZ,
    UNIQUE (author_id, prompt_id)
);
