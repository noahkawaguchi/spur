# Spur

## Concept

Spur is a full stack reply-based social platform. With no original posts, every post must instead be in reply to another post, creating a tree/DAG structure starting from the initial root post.

```
         [root post]         <-- Created artificially
        /     |     \
  [reply]  [reply]  [reply]  <-- Created by users
 /       \
[reply]  [reply]   (etc.)    <-- Created by users
```

## Stack

- **Backend**: Rust, [Axum](https://github.com/tokio-rs/axum), [SQLx](https://github.com/launchbadge/sqlx) (Postgres), and [more great crates](backend/Cargo.toml)
- **Frontend**: TypeScript, React, Vite, Vitest, React Testing Library, pnpm

## Features

- User authentication using bcrypt hashing and JSON Web Tokens
- Friendships between users
- Reading and writing posts
- Discovering new posts via parent/child relationships in the tree

