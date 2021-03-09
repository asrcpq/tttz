# TTTZ

## Require

* rust(cargo)

* a VT100 compatible terminal

## Features

* TUI with sound, cross-platform(maybe)

* Replay automatically saved(working)

* Dummy AI, support strategy mode(moves in turn)

## Special Rules

* Special keys: move left/right until fail(=long press in gui)

* SRS kick table(with 180 degree kick)

* Garbage in same attack has a 30%(or 40%) probability to shift

### Special Attack Table(working)

* All twists are rewarded(in different base attack), with mini-twists

* Combos are calculated as multipliers.

	single combos are extremely weakened(e.g, attack always 1 for 3-14 combo)

* B2B becomes twist combo multiplier(tcm)

	* A plain drop will reset twist combo(tc) to 1x(if it was not 0x)

	* A twist drop will increase tc

	* A regular clear will reset tc to 0x

* Attack Computation

	```
	b = base_attack[cleared_lines] = [0.5, 1.0, 2.0, 4.0]
	tb = twist_bonus[mini|regular][block_type]
	cm = 1 + combo * COMBO_INC=0.2
	tcm = 1 + tcm * TWIST_COMBO_INC=0.5
	atk = floor(b * tb * (cm + tcm))
	```

	`twist_bonus` table

	Block | S/Z/T | L/J | I
	--- | --- | --- | ---
	mini | 2 | 1.5 | N/A
	regular | 3 | 2 | 1.5

### Event-driven(instead of realtime)

* Zero delay, operations are applied as fast as possible.

* No gravity, infinite hold swap at any time

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

* \<Enter\> or \<EOF\> exit cmd mode

* `quit`
