#!/bin/sh

if ! [ -d "$PWD/world_server" ] || ! [ -d "$PWD/auth_server" ]; then
    echo "Please run start.sh from the root wrath-rs directory."
    exit 1
fi

parallel -- "cd auth_server && cargo run" "cd world_server && cargo run"
