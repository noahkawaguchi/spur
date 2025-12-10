####################################################################################################
# Settings/config
####################################################################################################

# Load a .env file if present
set dotenv-load := true

# The tag to use for the Postgres Docker image. Should match the one in the Compose files.
pg-tag := "18.0-alpine3.22"

####################################################################################################
# Dev containers/volume/network
#
# Requires the Docker CLI and a running VM (e.g. Docker Desktop, Colima) or equivalent.
####################################################################################################

dc-project := "docker compose -p spur"

# Start the full dev stack with migrations and seed data if necessary (this is the default recipe)
dc-up: img-build
    {{dc-project}} --profile init -f docker-compose.yml -f docker-compose.dev.yml up -d

# Stop the project's running containers
dc-stop:
    {{dc-project}} stop

# Remove the project's containers and volumes
[confirm("Are you sure you want to remove the containers and dev data?")]
dc-down:
    {{dc-project}} down --volumes

####################################################################################################
# Spur Docker image
#
# Pushing to GHCR requires being logged into the Docker CLI.
####################################################################################################

img-url := "ghcr.io/noahkawaguchi/spur:$SPUR_IMG_TAG"

# Build the Spur Docker image
img-build:
    docker build -t {{img-url}} .

# Push the Spur Docker image to GHCR, refusing to overwrite if the image/tag already exists
img-push:
    @if docker pull {{img-url}} >/dev/null 2>&1; then \
        echo "\nImage/tag already exists, refusing to overwrite: {{img-url}}\n"; \
        exit 1; \
    fi
    just img-build
    docker push {{img-url}}

####################################################################################################
# Pre-compilation step for compile time checked SQL queries without a live DB connection (needed for
# building in a Docker container)
#
# `sqlx` commands require the SQLx CLI with the feature flag for Postgres. TLS is not required to
# use the local development database in Docker. Therefore, the minimal sqlx-cli installation
# required for this project is: `cargo install sqlx-cli --no-default-features --features postgres`
####################################################################################################

prep-db-name := "spur_sqlx_prep_db"
prep-db-port := "55432"
prep-db-url := "postgres://$POSTGRES_USER:$POSTGRES_PASSWORD@localhost:" \
               + prep-db-port + "/$POSTGRES_DB"

# Update the `.sqlx` directory using an ephemeral Postgres container (after any query modifications)
sqlx-prep:
    if docker inspect {{prep-db-name}} >/dev/null 2>&1; then \
        docker rm -f {{prep-db-name}}; sleep 3; fi
    docker run --rm --name {{prep-db-name}} --env-file .env -p {{prep-db-port}}:5432 -d \
        postgres:{{pg-tag}}
    until docker exec {{prep-db-name}} pg_isready > /dev/null 2>&1; do sleep 1; done
    sqlx migrate run -D {{prep-db-url}}
    cargo sqlx prepare -D {{prep-db-url}} -- --workspace --all-targets --all-features
    docker stop {{prep-db-name}}

####################################################################################################
# Migrations
#
# Requires the Atlas CLI (https://atlasgo.io/getting-started#installation).
# Recipes that use an ephemeral DB require the VM for Docker to be running.
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
#
# The ephemeral Postgres DB requires the Docker CLI and a running Docker daemon.
####################################################################################################

test-db-name := "spur_test_db"
test-db-port := "2345"
test-db-url := "postgres://$POSTGRES_USER:$POSTGRES_PASSWORD@localhost:" \
               + test-db-port + "/$POSTGRES_DB"

# Run tests using an ephemeral Postgres container
test:
    if docker inspect {{test-db-name}} >/dev/null 2>&1; then \
        docker rm -f {{test-db-name}}; sleep 3; fi
    docker run --rm --name {{test-db-name}} --env-file .env -p {{test-db-port}}:5432 -d \
        postgres:{{pg-tag}}
    until docker exec {{test-db-name}} pg_isready > /dev/null 2>&1; do sleep 1; done
    DATABASE_URL={{test-db-url}} SQLX_OFFLINE=true cargo test --workspace --all-targets
    docker stop {{test-db-name}} > /dev/null 2>&1 &

# Manually stop the test database container
test-clean:
    docker stop {{test-db-name}}

# Generate and display test coverage (requires `cargo install cargo-llvm-cov`)
coverage:
    cargo llvm-cov --open

# Check spelling according to `cspell.json`. Requires `npm i -g cspell`.
spell-check:
    cspell .
