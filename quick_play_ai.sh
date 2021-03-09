#!/bin/sh
trap exittrap EXIT
exittrap() {
	pkill tttz
}
cargo run --release --bin tttz-server 2>server.log &
sleep 1
cargo run --release --bin tttz-tui \
execute "spawnai strategy" \
execute "sleep 1000" \
execute "request 2" \
2>client.log
