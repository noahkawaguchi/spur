CREATE TABLE post (
    id          INT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    author_id   INT REFERENCES users(id) ON DELETE SET NULL,
    parent_id   INT REFERENCES post(id) ON DELETE RESTRICT, -- Rows should only be soft deleted
    body        TEXT CHECK (trim(body) <> ''),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    edited_at   TIMESTAMPTZ,
    archived_at TIMESTAMPTZ,
    deleted_at  TIMESTAMPTZ,
    UNIQUE (author_id, parent_id) -- Enforce one child post per person per parent post
);

-- Enforce that only one post can have a NULL parent_id
CREATE UNIQUE INDEX one_root_post
    ON post ((true))
    WHERE parent_id IS NULL;
