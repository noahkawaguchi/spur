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
