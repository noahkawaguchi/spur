# Spur

## Concept

Spur is a prompt-based social writing platform. Inverting the standard setup of posts and replies, posts must instead be in response to someone else's prompt.

## Stack

- **Backend**: [Axum](https://github.com/tokio-rs/axum), [SQLx](https://github.com/launchbadge/sqlx) (Postgres), and [more great crates](spur-api/Cargo.toml)
- **CLI**: [Clap](https://github.com/clap-rs/clap), [Inquire](https://github.com/mikaelmello/inquire), and [more great crates](spur-cli/Cargo.toml)

## Features

| Action | CLI Command | API Endpoint |
| - | - |  - |
| Create a new account | `signup` | `POST` `/auth/signup` |
| Log in to an existing account | `login` | `POST` `/auth/login` |
| Confirm JSON Web Token validity | `check` | `GET` `/auth/check` |
| Add a friend by username | `add` | `POST` `/friends` |
| List your friends | `friends` | `GET` `/friends` |
| List pending friend requests to you | `requests` | `GET` `/friends/requests` |
| Create a new prompt | `prompt` | `POST` `/prompts` |
| Create a new post | `write` | `GET` `/prompts/{prompt_id}` then `POST` `/posts` |
| Read a friend's post | `read` | `GET` `/posts/{post_id}` |
| List prompts and posts from all your friends | `feed` | `GET` `/content` |
| List a specific friend's prompts and posts | `profile` | `GET` `/content/friend/{username}` |
| List your own prompts and posts | `me` | `GET` `/content/me` |
