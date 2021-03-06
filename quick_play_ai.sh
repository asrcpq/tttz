#!/bin/sh
trap exittrap EXIT
exittrap() {
	pkill mypuzzle
}
cargo run --release --bin mypuzzle-server 2>/dev/null &
sleep 1
cargo run --release --bin mypuzzle-client \
execute "aispawn" \
execute "sleep 1000" \
execute "request 2" \
2>/dev/null
