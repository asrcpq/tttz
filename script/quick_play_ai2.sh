#!/bin/sh
trap exittrap EXIT
exittrap() {
	pkill tttz
}
spd=300
if [ -n "$1" ]; then
	spd="$1"
fi
cd "$(dirname "$0")/.."
cargo run --release --bin tttz-server 2>/dev/shm/tttz-server.log &
sleep 1
cargo run --release --bin tttz-tui \
execute "spawnai algo cc strategy $spd" \
execute "sleep 300" \
execute "request 2" \
2>/dev/shm/tttz-client.log
