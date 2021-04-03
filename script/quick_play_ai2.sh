#!/bin/sh
trap exittrap EXIT
exittrap() {
	pkill tttz
}
algo="mm"
if [ -n "$1" ]; then
	algo="$1"
fi
cd "$(dirname "$0")/.."
cargo run --release --bin tttz-server 2>/dev/shm/tttz-server.log &
sleep 1
cargo run --release --bin tttz-tui \
execute "spawnai algo $algo strategy 300" \
execute "sleep 300" \
execute "request 2" \
2>/dev/shm/tttz-client.log
