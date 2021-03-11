#!/bin/sh
trap exittrap EXIT
exittrap() {
	pkill tttz
}
cargo run --release --bin tttz-server 2>/dev/shm/tttz-server.log &
sleep 1
cargo run --release --bin tttz-tui \
execute "spawnai algo sbai speed 200" \
execute "sleep 1000" \
execute "request 2" \
2>/dev/shm/tttz-client.log
