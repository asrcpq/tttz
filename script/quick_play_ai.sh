#!/bin/sh
trap exittrap EXIT
exittrap() {
	pkill tttz
}
cd "$(dirname "$0")/.."
cargo run --release --bin tttz-server 2>/dev/shm/tttz-server.log &
sleep 1
cargo run --release --bin tttz-tui \
execute "spawnai algo mm speed 500" \
execute "sleep 1000" \
execute "request 2" \
2>/dev/shm/tttz-client.log
