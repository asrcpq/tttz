#!/bin/sh
trap exittrap EXIT
exittrap() {
	pkill tttz
}
cargo run --release --bin tttz-server 2>server.log &
sleep 1
cargo run --release --bin tttz-tui \
execute "spawnai algo basic_cover speed 100" \
execute "sleep 50" \
execute "spawnai algo sbai speed 100" \
execute "sleep 300" \
execute "invite 2 3" \
execute "view 2" \
execute "panel 0 2" \
execute "view 3" \
execute "panel 1 3" \
execute "" \
2>client.log
