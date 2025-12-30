#!/usr/bin/env bash
set -euo pipefail

################################################################################
# Description: Minimal bootstrap script to set up and run the Docker Compose
#              stack in its production configuration using only Docker images
#              without the full repo.
#
# Requires:    wget and docker
#
# Usage:       See ./bootstrap.sh help
################################################################################

spur_img="ghcr.io/noahkawaguchi/spur:${SPUR_IMG_TAG:-latest}"

# Provides the user with a prompt asking if they "want to $1" and exits with
# status 1 if they respond with something other than "y" or "yes" (case
# insensitive).
confirm() {
  printf "Are you sure you want to %s? [y/N]: " "$1"
  read -r ans

  case "$ans" in
  [Yy] | [Yy][Ee][Ss]) return 0 ;;
  *)
    echo "Canceled" >&2
    exit 1
    ;;
  esac
}

# Downloads a file from the main Spur repo.
#
# Usage: download_file <file_path> [other_args]
#   file_path     The path to the file relative to the repository root.
#   other_args    Any other args to pass to wget in addition to the URL.
download_file() {
  local file_path="$1"
  shift
  wget "https://raw.githubusercontent.com/noahkawaguchi/spur/main/$file_path" \
    "$@"
}

# Pulls the Spur Docker image and starts the project's Docker Compose stack with
# the `init` profile.
pull_and_start() {
  docker pull "$spur_img"
  docker compose -p spur --profile init up -d
}

# Parse the command
case "$1" in

# Download only the required files
files)
  download_file docker-compose.yml

  download_file .env.example
  if [ ! -e .env ]; then cp .env.example .env; fi

  mkdir -p caddy_conf
  download_file caddy_conf/Caddyfile -O caddy_conf/Caddyfile

  printf "\nNext fill out .env based on .env.example, then execute %s run\n\n" \
    "$0"
  ;;

# Start the Compose stack
run) pull_and_start ;;

# Destroy and remake the database
reset)
  confirm "destroy and remake the database"
  docker compose -p spur stop
  docker compose -p spur rm -sf db
  docker volume rm spur_pg_data
  pull_and_start
  ;;

# Help message
*)
  echo "Usage: $0 COMMAND

Commands:
  files   Download the required files from the main Spur repo
  run     Pull the Spur Docker image and run the Docker Compose stack
  reset   Destroy and remake the database

Use the SPUR_IMG_TAG environment variable to specify a tag other than \"latest\"
to use for the Spur Docker image.

Remember to fill out .env between \`files\` and \`run\`.
"
  ;;

esac
