CREATE TABLE users (
    id INT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    name TEXT NOT NULL CHECK (trim(name) <> ''),
    email TEXT NOT NULL CONSTRAINT users_email_unique UNIQUE CHECK (trim(email) <> ''),
    username TEXT NOT NULL CONSTRAINT users_username_unique UNIQUE CHECK (trim(username) <> ''),
    password_hash TEXT NOT NULL CHECK (trim(password_hash) <> ''),
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
