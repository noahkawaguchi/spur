# Spur

## Concept

Spur is a full stack prompt-based social writing platform. Inverting the standard setup of posts and replies, posts must instead be in response to someone else's prompt.

## Stack

- **Backend**: Rust, [Axum](https://github.com/tokio-rs/axum), [SQLx](https://github.com/launchbadge/sqlx) (Postgres), and [more great crates](backend/Cargo.toml)
- **Frontend**: TypeScript, React, Vite, Vitest, pnpm

## Features

- User authentication using bcrypt hashing and JSON Web Tokens
- Friendships between users
- Reading and writing posts and prompts

