-- Add migration script here
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL CHECK (trim(name) <> ''),
    email TEXT NOT NULL UNIQUE CHECK (trim(email) <> ''),
    username TEXT NOT NULL UNIQUE CHECK (trim(username) <> ''),
    password_hash TEXT NOT NULL CHECK (trim(password_hash) <> ''),
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
