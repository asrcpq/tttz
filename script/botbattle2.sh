#!/bin/sh
trap exittrap EXIT
exittrap() {
	pkill tttz
}
a="mm"
if [ -n "$1" ]; then
	a="$1"
fi
cd "$(dirname "$0")/.."
cargo run --release --bin tttz-server 2>/dev/shm/tttz-server.log &
sleep 1
cargo run --release --bin tttz-tui \
execute "spawnai algo sbai strategy_initiator 40" \
execute "spawnai algo $a strategy 40" \
execute "sleep 300" \
execute "view 2" \
execute "panel 0 2" \
execute "panel 1 3" \
execute "invite 2 3" \
execute "" \
2>/dev/shm/tttz-client.log
