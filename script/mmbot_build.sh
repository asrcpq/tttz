#!/bin/bash
cd "$(dirname "$0")/.."
mkdir -p thirdparty && cd thirdparty
if [ -d MisaMinoBot ]; then
	echo "mmbot folder found, exiting"
	exit 1
fi
git clone https://github.com/asrcpq/MisaMinoBot.git
cd MisaMinoBot/tetris_ai
make -f Makefile CONF=Release -j
mkdir -p ~/.local/bin
cp dist/Release/GNU-Linux/tetris_ai ~/.local/bin/tttz_mmbot
