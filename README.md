# TTTZ

## Require

* rust(cargo)

* a VT100 compatible terminal

## Features

* TUI, cross-platform(works needed)

* Replay automatically saved(working)

* Dummy AI, support strategy mode(moves in turn)

## Special Rules

* Special keys: move left/right until fail(=long press in other implements)

	Because terminals don't have keydown event.

* SRS kick table(with 180 degree kick)

* Garbage in same attack has a 30%(or 40%) probability to shift

### Special attack table(working)

* All twists are rewarded(with different weight)

* Combo and b2b bonus are calculated as multipliers.

### Event-driven(instead of realtime)

* Zero delay, operations are applied as fast as possible.

* No gravity(blocks can move down), infinite hold swap

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
