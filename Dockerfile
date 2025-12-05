###################################################
# Builder stage
###################################################

# This base image's default target triple for Rust, {architecture here}-unknown-linux-musl, is
# already the one desired in the runtime stage, so there's no need to specify a different target
# when compiling
FROM rust:1.91.0-alpine3.22 AS builder

# Install C headers and static libraries needed by dependencies in the builder stage
RUN apk add --no-cache musl-dev

WORKDIR /app

# Pre-build dependencies so they're cached even if the app code changes
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release && rm -rf src

# Explicitly copy only the directories that are needed
COPY migrations migrations
COPY .sqlx .sqlx
COPY src src

# Compile the main app and three helper binaries, requiring Cargo.lock and dependencies cached
RUN cargo build --release --frozen --bins

###################################################
# Runtime stage
###################################################

FROM scratch

# Copy the main app and three helper binaries from the builder stage
COPY --from=builder /app/target/release/spur /spur
COPY --from=builder /app/target/release/migrate /migrate
COPY --from=builder /app/target/release/seed /seed
COPY --from=builder /app/target/release/healthcheck /healthcheck

# Use the tiny healthcheck binary for checking if the server is running
HEALTHCHECK --interval=15s --timeout=1s --start-period=5s --retries=3 CMD ["/healthcheck"]

# Switch to non-root
USER 1000:1000

# The server itself is the entrypoint because there is no shell.
# Override the entrypoint to run one of the other binaries.
ENTRYPOINT ["/spur"]
