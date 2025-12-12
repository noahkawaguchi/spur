####################################################################################################
#
# Using this justfile requires the command runner `just`, the Docker CLI, and a running Docker
# daemon. Other dependencies are documented below with the recipes that need them.
#
####################################################################################################

####################################################################################################
# Settings/config
####################################################################################################

# Load a .env file if present
set dotenv-load := true

# The tag to use for the Postgres Docker image. Should match the one used in `docker-compose.yml`.
pg-tag := "18.0-alpine3.22"

####################################################################################################
# Main Docker Compose stack
####################################################################################################

spur-img-tag := env("SPUR_IMG_TAG", "latest")
dc-project := "docker compose -p spur -f docker-compose.yml -f docker-compose.dev.yml" \
              + " --profile init"

# Build/start the Compose stack with migrations and seed data if necessary (the default recipe)
dc-up:
    docker build -t ghcr.io/noahkawaguchi/spur:{{spur-img-tag}} .
    {{dc-project}} up -d

# Stop the project's running containers
dc-stop:
    {{dc-project}} stop

# Remove the project's containers and volumes
[confirm("Are you sure you want to remove the containers and dev data?")]
dc-down:
    {{dc-project}} down --volumes

####################################################################################################
# Pre-compilation step for compile time checked SQL queries without a live DB connection (needed for
# building in a Docker container)
#
# `sqlx` commands require the SQLx CLI with the feature flag for Postgres. TLS is not required to
# use the local development database in Docker. Therefore, the minimal sqlx-cli installation
# required for this project is: `cargo install sqlx-cli --no-default-features --features postgres`
####################################################################################################

# Update the `.sqlx` directory using an ephemeral Postgres container (after any query modifications)
sqlx-prep: temp-db-start && temp-db-stop
    sqlx migrate run -D {{temp-db-url}}
    cargo sqlx prepare -D {{temp-db-url}} -- --workspace --all-targets --all-features

####################################################################################################
# Migrations
#
# Requires the Atlas CLI (https://atlasgo.io/getting-started#installation).
####################################################################################################

# A URL to pass to Atlas so that it can create an ephemeral DB to work in
ephemeral-pg := "docker://postgres/" + pg-tag + "/dev"

# The master schema to be edited by hand and diffed by Atlas
schema-file := "file://schema.sql"

# Recompute `atlas.sum` after manual changes
mg-hash:
    atlas migrate hash

# Validate all migrations
mg-validate:
    atlas migrate validate --dev-url {{ephemeral-pg}}

# Generate a new migration file by diffing the schema
migration name:
    atlas migrate diff {{name}} --to {{schema-file}} --dev-url {{ephemeral-pg}}

####################################################################################################
# Testing and code quality
####################################################################################################

# Run tests using an ephemeral Postgres container
test: temp-db-start
    DATABASE_URL={{temp-db-url}} SQLX_OFFLINE=true cargo test --workspace --all-targets; \
    status=$?; \
    just temp-db-stop; \
    exit $status

# Generate and display test coverage (requires `cargo install cargo-llvm-cov`)
coverage: temp-db-start && temp-db-stop
    DATABASE_URL={{temp-db-url}} SQLX_OFFLINE=true cargo llvm-cov --open --workspace --all-targets

# Check spelling according to `cspell.json`. Requires `npm i -g cspell`.
spell-check:
    cspell .

####################################################################################################
# Ephemeral Postgres container helper utilities
#
# These recipes are hidden because they are primarily meant to be used by other recipes as
# dependencies, but they can be called directly if necessary.
####################################################################################################

temp-db-name := "spur_temp_db"
temp-db-port := env("SPUR_TEMP_DB_PORT", "55432")
temp-db-url := "postgres://$POSTGRES_USER:$POSTGRES_PASSWORD@localhost:" \
               + temp-db-port + "/$POSTGRES_DB"

# Start an ephemeral Postgres container and wait until it's ready
[private]
temp-db-start:
    if docker inspect {{temp-db-name}} >/dev/null 2>&1; then \
        docker rm -f {{temp-db-name}}; sleep 3; fi
    docker run --rm --name {{temp-db-name}} --env-file .env -p {{temp-db-port}}:5432 -d \
        postgres:{{pg-tag}}
    until docker exec {{temp-db-name}} pg_isready > /dev/null 2>&1; do sleep 1; done

# Stop the ephemeral Postgres container (also removing it)
[private]
temp-db-stop:
    docker stop {{temp-db-name}}
