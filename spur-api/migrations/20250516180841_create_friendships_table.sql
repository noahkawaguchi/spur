-- Add migration script here
CREATE TABLE friendships (
    first_id INTEGER NOT NULL,
    second_id INTEGER NOT NULL,
    requester_first BOOLEAN NOT NULL,
    confirmed BOOLEAN NOT NULL DEFAULT FALSE,
    requested_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    confirmed_at TIMESTAMPTZ,
    PRIMARY KEY (first_id, second_id),
    CHECK (first_id < second_id),
    FOREIGN KEY (first_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (second_id) REFERENCES users(id) ON DELETE CASCADE
);
