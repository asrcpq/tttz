# TTTZ

## Require

* rust(cargo)

* a VT100 compatible terminal

## Features

* TUI, cross-platform(maybe)

* Replay automatically saved(working)

* A greedy search AI, also support strategy mode(moves in turn)

## Special Rules

* SRS kick table(with 180 degree kick), with tspin, b2b and pc implemented.

* Garbage in same attack has a 30%(or 40%) probability to shift

* Event-driven(instead of realtime, details below)

### Event-driven features

* Zero delay, operations are applied as fast as possible.

* No gravity(blocks can move down), infinite hold swap

	This means the board won't change when there's no input.

* Pending Attack won't apply until

	* a drop without line clear will drain the pending garbages

	* max 5 attacks pending, or early attacks will pop

* Block generate at top

	Precisely, at y=38(bottom = 0) to prevent wall kick.

* Die, if after board change shadow block is totally invisible

	There are two types of board changes: hard drop and garbage overflow.

## Gameplay

* h/l: move left/right

* H/L: move left/right until fail

* k: hard drop

* j: soft drop

* K: move down 5

* J: move down 1

* Space: hold

* r: suicide/restart

* /: enter cmd mode

## Commands:

* `spawnai strategy` or `spawnai speed [sleep_millis]`

* `kick <id>`

* `clients` list clients

* `myid`

* `free` enter single play mode

* `request <id>` send battle request to id(ai will immediately accept)

* `accept <id>` accept battle request from id

* `pair` start pairing

* \<Enter\> exit cmd mode

* `quit`
