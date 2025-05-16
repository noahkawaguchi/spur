-- Add migration script here
CREATE TABLE friendships (
    first_user_id INTEGER NOT NULL,
    second_user_id INTEGER NOT NULL,
    requester_id INTEGER NOT NULL,
    confirmed BOOLEAN NOT NULL DEFAULT FALSE,
    requested_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    confirmed_at TIMESTAMPTZ,
    PRIMARY KEY (first_user_id, second_user_id),
    CHECK (first_user_id < second_user_id),
    CHECK (requester_id IN (first_user_id, second_user_id)),
    FOREIGN KEY (first_user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (second_user_id) REFERENCES users(id) ON DELETE CASCADE
);
