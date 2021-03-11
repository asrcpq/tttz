#!/bin/sh
trap exittrap EXIT
exittrap() {
	pkill tttz
}
cargo run --release --bin tttz-server 2>/dev/shm/tttz-server.log &
sleep 1
cargo run --release --bin tttz-tui \
execute "free" \
2>/dev/shm/tttz-client.log
