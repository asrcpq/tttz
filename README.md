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

* All twists are rewarded(in different base attack), with mini-twists

* Combos are calculated as multipliers.

	single combos are extremely weakened(e.g, attack always 1 for 3-14 combo)

* b2b becomes twist combo multiplier(tcm)

	* A plain drop will reset twist combo(tc) to 1x(if it was not 0x)

	* A twist drop will increase tc

	* A regular clear will reset tc to 0x

* Attack Calculation

	```
	b = base_attack[cleared_lines] = [0.4, 1.0, 2.0, 4.0]
	tb = twist_bonus[mini|regular][cleared_lines]
	cm = 1 + combo * COMBO_INC=0.2
	tcm = 1 + tcm * TWIST_COMBO_INC=0.5
	atk = round(b * tb * (cm + tcm))
	```

	Value of `twist_bonus`

	Block | S/Z/T | L/J | I
	--- | --- | --- | ---
	mini | 2 | 1.5 | N/A
	regular | 3 | 2 | 1.5

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
