#!/bin/sh
spd=300
if [ -n "$1" ]; then
	spd="$1"
fi
cd "$(dirname "$0")/.."
cargo run --release --bin tttz-tui \
execute "spawnai algo sbai speed $spd" \
execute "sleep 300" \
execute "request 2" \
2>/dev/shm/tttz-client.log
