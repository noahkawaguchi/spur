# Spur

### 🔗 Live link: [spur.noahkawaguchi.com](https://spur.noahkawaguchi.com/)

## Table of Contents

1. [Concept](#concept)
2. [Tech Stack](#tech-stack)
3. [Design](#design)
4. [Testing](#testing)
5. [Security and Safety](#security-and-safety)
6. [Development and Deployment](#development-and-deployment)

## Concept

Spur is a reply-based social platform. With no original posts, every post must instead be in reply to another post, creating a tree/DAG structure starting from the initial root post.

```
         [root post]         <-- Created artificially
        /     |     \
  [reply]  [reply]  [reply]  <-- Created by users
 /       \
[reply]  [reply]   (etc.)    <-- Created by users
```

## Tech Stack

|                               |                                        |
| ----------------------------- | -------------------------------------- |
| **_Languages_**               | Rust, SQL (PostgreSQL)                 |
| **_Main Crates (Libraries)_** | Axum, SQLx, Utoipa                     |
| **_Key Tools_**               | Docker, Caddy                          |
| **_Testing_**                 | Rust's native test framework & Mockall |
| **_Deployment_**              | AWS EC2                                |

## Design

The project's design focuses on layers/modules being decoupled via traits (interfaces). Core business rules are defined in the `domain` module, while concrete implementations are in the `infra` module. Domain services handle logic that is specific to a single domain, application services handle read/write logic that spans multiple domains, and read models handle read-only logic that spans multiple domains.

The server exposes a REST API with OpenAPI documentation and a built-in Swagger UI for interactive exploration. All components of the design are containerized in a Docker Compose stack on AWS EC2.

```
              Caddy (provides HTTPS)
                       |
               Swagger UI (Utoipa)
                       |
                API layer (Axum)
               /       |        \
domain services   app services   \
      \            /              \
       repositories          read models
              \                   /
               PostgreSQL database
```

## Testing

The project is tested with Rust's native test framework and Mockall in the Tokio runtime.

- Run tests: `cargo test` (The development database must be running.)
- Show coverage: `just coverage`

Due to the decoupled design, the testing strategy focuses on dependency injection using a combination of automatic mocks from the `mockall` crate and manual mocks. SQL schemas and queries are tested using ephemeral test databases in Docker.

## Security and Safety

- All secrets are read in via environment variables.
- The app can only be accessed over HTTPS.
- The Docker Compose stack runs rootless to avoid the security concerns of running Docker as root. The binaries inside the app containers also run rootless.
- The containers in the stack are exposed to each other only through an internal Docker network, not localhost. All communications outside this network must first go through the Caddy container, which provides HTTPS.
- HTTP requests that access user data must have the standard Authorization Bearer header with a valid JSON Web Token.
- The main server binary and the three helper binaries contain no `unsafe`, `unwrap`, or `expect`. This is enforced using lints set to the `forbid` level. (`unwrap` and `expect` are allowed in tests.)

## Development and Deployment

- The project's main language is Rust, which can be installed via `rustup` as documented [here](https://rust-lang.org/tools/install/).
- `just` is a command runner that can be installed via one of the methods described [here](https://github.com/casey/just/blob/master/README.md#installation).
- Common commands used in development and any additional tools they require are documented (and automated) via the `justfile`.
- The `.env.example` file describes the necessary environment variable configurations.
- The project is deployed as described in [deploy/steps.md](deploy/steps.md).
