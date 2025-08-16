.PHONY: db-up db-rm migrate-add migrate-run

# Initialize DB
db-up:
	docker compose -p spur up -d

# Stop services and remove volumes (delete data)
db-rm:
	docker compose down -v

# Create a new migration
migrate-add:
	sqlx migrate add $(word 2, $(MAKECMDGOALS))

# Run pending migrations
migrate-run:
	sqlx migrate run

# dummy rule to absorb arguments
%:
	@:
