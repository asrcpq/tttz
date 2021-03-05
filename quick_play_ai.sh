#!/bin/sh
cargo run --release --bin mypuzzle-client \
execute "aispawn" \
execute "sleep 1000" \
execute "request 2" \
2>mypuzzle-client.log
