CREATE TABLE friendships (
    first_id INT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    second_id INT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    requester_first BOOL NOT NULL,
    confirmed BOOL NOT NULL DEFAULT FALSE,
    requested_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    confirmed_at TIMESTAMPTZ,
    PRIMARY KEY (first_id, second_id),
    CHECK (first_id < second_id)
);
