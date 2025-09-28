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
