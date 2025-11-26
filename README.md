# Spur

### 🔗 Live link: [spur.noahkawaguchi.com](https://spur.noahkawaguchi.com/)

## Table of Contents

1. [Concept](#concept)
2. [Tech Stack](#tech-stack)
3. [Design](#design)
4. [Testing](#testing)
5. [Security and Safety](#security-and-safety)
6. [Development and Deployment](#development-and-deployment)
7. [Demo Videos](#demo-videos)

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
- The main backend server binary and the three helper binaries contain no `unsafe`, `unwrap`, or `expect`. This is enforced using lints set to the `forbid` level. (`unwrap` and `expect` are allowed in tests.)
- Type safety is enforced in the frontend via ESLint rules forbidding `any` and type assertions (`as`).

## Development and Deployment

- The backend uses Rust, which can be installed via `rustup` as documented [here](https://rust-lang.org/tools/install/).
- `just` is a command runner used primarily in the backend. It can be installed via one of the methods described [here](https://just.systems/man/en/). Additional development tools required by some recipes are documented in the `justfile`s.
- The frontend uses Node, which can be installed via one of the methods described [here](https://nodejs.org/en/download). `pnpm` can then be installed with `npm i -g pnpm`.
- Both the backend and the frontend have `.env.example` files that describe the necessary environment variable configurations.
- Common commands used in development are documented and automated via the `justfile` in the backend and `package.json` in the frontend.
- The frontend is deployed via `pnpm run deploy`, and the backend is deployed as described in [backend/deploy/steps.md](backend/deploy/steps.md).

## Demo Videos

_(All videos recorded using the real deployed app)_

### User authentication using bcrypt hashing and JSON Web Tokens

Log in to an existing account

https://github.com/user-attachments/assets/a9acb27e-2c3c-4949-8092-a5cb25edff16

Create a new account

https://github.com/user-attachments/assets/648c0fa2-09f3-4c64-95df-b45d6923ebd0

### Send and accept friend requests

https://github.com/user-attachments/assets/f1717e8a-63b9-4075-9562-a4f97e6a836d

https://github.com/user-attachments/assets/8d07fd44-3bef-45a8-9227-f57d5947607c

### Read and reply to posts

https://github.com/user-attachments/assets/99939975-6e8c-414d-b60e-5eec8b5034f9

### Discover new posts via parent/child relationships in the tree

https://github.com/user-attachments/assets/ff2c6a40-f763-47d1-9719-4580fc19ed5b

There are also various business rules implemented, such as not replying to one's own post and not replying to the same post more than once.
