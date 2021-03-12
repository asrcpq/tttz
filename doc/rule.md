# Rules

* Special keys: move left/right until fail(=long press in gui)

* SRS kick table(with 180 degree kick)

* Garbage in same attack has a 30%(or 40%) probability to shift

## Special Attack Table

* All twists are rewarded(in different base attack), with mini-twists

* Combos are calculated as multipliers.

* B2B becomes twist combo multiplier(tcm)

	* A plain drop will reset twist combo(tc) to 1x(if it was not 0x)

	* A twist drop will increase tc

	* A regular clear will reset tc to 0x

* Attack Computation

	```
	b = base_attack[cleared_lines] = [0.5, 1.5, 2.5, 4.0]
	tb = twist_bonus[mini|regular][block_type]
	cm = 1 + combo * COMBO_INC=0.3
	tcm = 1 + tcm * TWIST_COMBO_INC=0.6
	atk = floor(b * tb * (cm + tcm))
	```

	`twist_bonus` table

	Block | T | S/Z | L/J | I
	--- | --- | --- | --- | ---
	mini | 2 | 2 | 1.5 | N/A
	regular | 3 | 2.5 | 2 | 1.5

## Event-driven(instead of realtime)

* Zero delay, operations are applied as fast as possible.

* No gravity, infinite hold swap at any time

* Pending Attack won't apply until

	* a drop without line clear will drain the pending garbages

	* max 5 attacks pending, or early attacks will pop

* Block generate at top

	Precisely, at y=38(bottom = 0) to prevent wall kick.

* Die, if after board change shadow block is totally invisible

	There are two types of board changes: hard drop and garbage overflow.
