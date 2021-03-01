trap killproc EXIT
killproc() {
	pkill mypuzzle
}
cargo build --release
cargo run --release --bin mypuzzle-server &
cargo run --release --bin mypuzzle-ai1 sleep 50 &
cargo run --release --bin mypuzzle-ai1 sleep 50 &
