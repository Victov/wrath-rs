#!/bin/sh

export WRATH_RS_DIR="$PWD"

if ! [ -x "$(command -v docker-compose)" ]; then
    echo 'ERROR: docker-compose is not installed.' >&2
    exit 1
fi

if ! [ -d "$PWD/world_server" ] || ! [ -d "$PWD/auth_server" ]; then
    echo "Please run docker/docker-setup.sh from the root wrath-rs directory."
    exit 1
fi

# Check if SQLX CLI is installed.
if ! cargo install --list | grep sqlx-cli > /dev/null 2>&1; then
    echo "Installing sqlx-cli automatically..."
    if ! cargo install sqlx-cli; then
        echo "ERROR: automatic installation of sqlx-cli failed!"
        echo "Please follow the instructions at https://github.com/launchbadge/sqlx/tree/master/sqlx-cli"
        exit 1
    fi
    echo "Installation of sqlx-cli successful!"
fi

# Check if the user has already setup a docker container for wrath-rs, and forces the arg "--wipe" to be set before continuing.
if docker volume ls | grep wrath-rs_wrath-rs-database > /dev/null 2>&1; then
    if ! [ $# -gt 0 ] && ! [ "$1" = "--wipe" ]; then
        echo "WARNING: There is already a pre-existing wrath-rs DB volume."
        echo "Re-run docker-setup.sh with the '--wipe' arg if you wish to remove it and re-create the DB."
        echo "THIS WILL DELETE ALL DATA FROM THE DB."
        exit 1
    fi
fi

# Prompt the user for a default ROOT password.
echo "Please enter a ROOT password for your MariaDB container:"
read root_password

# Change the MariaDB root password in the docker-compose file.
sed -i "s/MARIADB_ROOT_PASSWORD:.*/MARIADB_ROOT_PASSWORD: $root_password/g" docker-compose.yml

# Setup .env files for the world/auth servers, pointing at the docker container.
cp auth_server/.env.template auth_server/.env
sed -E -i "s/AUTH_DATABASE_URL=.+/AUTH_DATABASE_URL=\"mysql:\/\/root:$root_password@localhost\/wrath_auth\"/g" auth_server/.env
sed -E -i "s/DATABASE_URL=.+/DATABASE_URL=\"mysql:\/\/root:$root_password@localhost\/wrath_auth\"/g" databases/wrath-auth-db/.env

cp world_server/.env.template world_server/.env
sed -E -i "s/AUTH_DATABASE_URL=.+/AUTH_DATABASE_URL=\"mysql:\/\/root:$root_password@localhost\/wrath_auth\"/g" world_server/.env
sed -E -i "s/REALM_DATABASE_URL=.+/REALM_DATABASE_URL=\"mysql:\/\/root:$root_password@localhost\/wrath_realm\"/g" world_server/.env
sed -E -i "s/DATABASE_URL=.+/DATABASE_URL=\"mysql:\/\/root:$root_password@localhost\/wrath_realm\"/g" databases/wrath-realm-db/.env

# Delete any pre-existing DB volumes.
docker-compose down
docker volume rm wrath-rs_wrath-rs-database

# Bring up the DB.
if ! docker-compose up -d db; then
    echo "Failed to bring up the docker DB!"
    exit 1
fi

# Run the migrations in each DB folder.
cd "$WRATH_RS_DIR/databases/wrath-auth-db"
cargo sqlx database drop
cargo sqlx database create
cargo sqlx migrate run

cd "$WRATH_RS_DIR/databases/wrath-realm-db"
cargo sqlx database drop
cargo sqlx database create
cargo sqlx migrate run
