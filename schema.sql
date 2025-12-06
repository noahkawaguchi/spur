-- Reusable type for non-empty text columns
CREATE DOMAIN non_empty_text AS TEXT
    CONSTRAINT text_non_empty CHECK (VALUE ~ '\S'); -- At least one non-whitespace character

CREATE TABLE users (
    id            INT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    name          non_empty_text NOT NULL,
    email         non_empty_text NOT NULL CONSTRAINT users_email_unique UNIQUE,
    username      non_empty_text NOT NULL
                      CONSTRAINT users_username_chars CHECK (username ~ '^[A-Za-z0-9_-]+$')
                      CONSTRAINT users_username_unique UNIQUE,
    password_hash non_empty_text NOT NULL,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE friendship (
    -- The IDs of the two users, always ordered
    lesser_id        INT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    greater_id       INT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    -- Whether the user with the lesser ID is the one who initiated the request
    lesser_requested BOOL NOT NULL,
    requested_at     TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    confirmed_at     TIMESTAMPTZ,
    PRIMARY KEY (lesser_id, greater_id),
    CONSTRAINT friendship_id_ordering CHECK (lesser_id < greater_id)
);

CREATE TABLE post (
    id          INT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    author_id   INT REFERENCES users(id) ON DELETE SET NULL,
    parent_id   INT REFERENCES post(id) ON DELETE RESTRICT, -- Rows should only be soft deleted
    body        non_empty_text,
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

