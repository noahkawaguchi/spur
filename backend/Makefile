.PHONY: db-up db-rm migrate-add migrate-run migrate-clean db-reset coverage


# Initialize DB
db-up:
	docker compose -p spur up -d

# Stop services and remove volumes (delete data)
db-rm:
	docker compose down -v

#
# `sqlx` commands require the SQLx CLI with the feature flag for Postgres.
# TLS is not required to use the local development database in Docker.
#
# Minimal installation required for this project:
# `cargo install sqlx-cli --no-default-features --features postgres`
#

# Create a new migration
migrate-add:
	sqlx migrate add $(word 2, $(MAKECMDGOALS))

# Run pending migrations
migrate-run:
	sqlx migrate run

# (Interactively) remove some or all migration files
migrate-clean:
	rm -i migrations/*

# Drop, recreate, and run migrations on the DB
db-reset:
	sqlx database reset

# Test coverage (requires `cargo install cargo-llvm-cov`)
coverage:
	cargo llvm-cov --open

# Dummy rule to absorb arguments
%:
	@:
