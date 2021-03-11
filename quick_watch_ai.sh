#!/bin/sh
trap exittrap EXIT
exittrap() {
	pkill tttz
}
if [ -z "$1" ]; then
	t=30
else
	t=$1
fi
cargo run --release --bin tttz-server 2>/dev/shm/tttz-server.log &
sleep 1
cargo run --release --bin tttz-tui \
execute "spawnai algo basic speed $t" \
execute "sleep 50" \
execute "spawnai algo sbai speed $t" \
execute "sleep 300" \
execute "invite 2 3" \
execute "view 2" \
execute "panel 0 2" \
execute "view 3" \
execute "panel 1 3" \
execute "" \
2>/dev/shm/tttz-client.log
