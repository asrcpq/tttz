trap killproc EXIT
killproc() {
	pkill mypuzzle
}
cargo build --release
cargo run --release --bin mypuzzle-server 2>"/dev/null" &
sleep 1
cargo run --release --bin mypuzzle-client \
execute "aispawn" \
execute "sleep 1000" \
execute "request 2" \
2>mypuzzle-client.log
