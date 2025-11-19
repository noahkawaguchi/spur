# Spur

### 🔗 Live link: [spur.noahkawaguchi.com](https://spur.noahkawaguchi.com/)

## Table of Contents

1. [Concept](#concept)
2. [Tech Stack](#tech-stack)
3. [Features](#features)
4. [Design](#design)
5. [Testing](#testing)
6. [Security and Safety](#security-and-safety)
7. [Development and Deployment](#development-and-deployment)

## Concept

Spur is a full-stack reply-based social platform. With no original posts, every post must instead be in reply to another post, creating a tree/DAG structure starting from the initial root post.

```
         [root post]         <-- Created artificially
        /     |     \
  [reply]  [reply]  [reply]  <-- Created by users
 /       \
[reply]  [reply]   (etc.)    <-- Created by users
```

## Tech Stack

|                             | Backend                      | Frontend                      |
| --------------------------- | ---------------------------- | ----------------------------- |
| **_Languages_**             | Rust, SQL (PostgreSQL)       | TypeScript                    |
| **_Main Crates/Libraries_** | Axum, SQLx                   | React                         |
| **_Key Tools_**             | Docker, Caddy                | Vite, pnpm                    |
| **_Testing_**               | Rust's native test framework | Vitest, React Testing Library |
| **_Deployment_**            | AWS EC2                      | GitHub Pages                  |

## Features

### _(Demonstrations coming soon)_

- User authentication using bcrypt hashing and JSON Web Tokens
- Friendships between users
- Reading and writing posts
- Discovering new posts via parent/child relationships in the tree

## Design

This project uses a decoupled design in which the backend and frontend can be developed and deployed independently. The backend exposes a REST API that the frontend consumes via HTTP requests.

```
  user interactions with React forms/buttons/etc.     \
                       |                              |  GitHub Pages
 HTTP requests via useRequest, a custom React hook    /
                       |
             Caddy (provides HTTPS)                  \
                       |                              \
            backend API layer (Axum)                  |
            /          |           \                  |  Docker
domain services    app services     \                 |  containers
      \            /                 \                |  on AWS EC2
       repositories              read models          |
              \                   /                   /
               PostgreSQL database                   /
```

The layers/modules of the backend are also decoupled via traits (interfaces). Core business rules are defined in the `domain` module, while concrete implementations are in the `infra` module. Domain services handle logic that is specific to a single domain, application services handle read/write logic that spans multiple domains, and read models handle read-only logic that spans multiple domains.

## Testing

The backend is tested with Rust's native test framework (in the `tokio` runtime).

- Run tests: `cargo test` (The development database must be running.)
- Show coverage: `just coverage`

Due to the decoupled design, the backend testing strategy focuses on dependency injection using a combination of automatic mocks from the `mockall` crate and manual mocks. SQL schemas and queries are tested using ephemeral test databases in Docker.

The frontend is tested with Vitest and React Testing Library.

- Run tests: `pnpm test`
- Show coverage: `pnpm coverage`

## Security and Safety

- All secrets are read in via environment variables.
- Both the backend and the frontend can only be accessed over HTTPS.
- The backend Docker Compose stack runs rootless to avoid the security concerns of running Docker as root. The binaries inside the app containers also run rootless.
- The containers in the stack are exposed to each other only through an internal Docker network, not localhost. All communications outside this network must first go through the Caddy container, which provides HTTPS.
- The backend's CORS policy specifically allows only the frontend URL.
- HTTP requests that access user data must have the standard Authorization Bearer header with a valid JSON Web Token.
- The main backend server binary and the three helper binaries are all written entirely in safe Rust, enforced using `#![forbid(unsafe_code)]`.
- Type safety is enforced in the frontend via ESLint rules forbidding `any` and type assertions (`as`).

## Development and Deployment

- The backend uses Rust, which can be installed via `rustup` as documented [here](https://rust-lang.org/tools/install/).
- `just` is a command runner used primarily in the backend. It can be installed via one of the methods described [here](https://just.systems/man/en/). Additional development tools required by some recipes are documented in the `justfile`s.
- The frontend uses Node, which can be installed via one of the methods described [here](https://nodejs.org/en/download). `pnpm` can then be installed with `npm i -g pnpm`.
- Both the backend and the frontend have `.env.example` files that describe the necessary environment variable configurations.
- Common commands used in development are automated via `package.json` in the frontend and the `justfile` in the backend.
- The frontend is deployed via `pnpm run deploy`, and the backend is deployed as described in [backend/deploy/steps.md](backend/deploy/steps.md).
