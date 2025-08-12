CREATE TABLE post (
    id          INT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    author_id   INT REFERENCES users(id) ON DELETE SET NULL,
    parent_id   INT REFERENCES post(id) ON DELETE RESTRICT, -- Rows should only be soft deleted
    body        TEXT CHECK (body IS NULL OR length(btrim(body)) > 0),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    edited_at   TIMESTAMPTZ,
    archived_at TIMESTAMPTZ,
    deleted_at  TIMESTAMPTZ,
    -- Enforce one child post per user per parent post
    CONSTRAINT post_author_parent_unique UNIQUE (author_id, parent_id)
);

-- Enforce that only one post can have a NULL parent_id
CREATE UNIQUE INDEX one_root_post
    ON post ((true))
    WHERE parent_id IS NULL;
