trap killproc EXIT
killproc() {
	pkill mypuzzle
}
cargo build --release
cargo run --release --bin mypuzzle-server 2>"/dev/null" &
sleep 1
cargo run --release --bin mypuzzle-client 2>mypuzzle-client.log
