# Rules

* Special keys: move left/right until fail(=long press in gui)

* SRS(with 180, asymmetric I kick) kick table

* Garbage in same attack has a 40% probability to shift,

## Special Attack Table

* All twists are rewarded equally, with mini-twists

* Combos are calculated as multipliers(cm).

* B2B becomes twist combo multiplier(tcm)

	* A plain drop will reset twist combo(tc) to 1x(if it was not 0x)

	* A twist drop will increase tc

	* A regular clear will reset tc to 0x

	* quad does not increase tcm

* Attack Computation

	```
	b = base_attack[cleared_lines] = [0.5, 1.5, 2.5, 4.0]
	tb = twist_bonus[mini|regular]
	cm = combo * COMBO_INC=0.3
	tcm = tc * TWIST_COMBO_INC=0.3
	atk = floor(b * tb * (cm + tcm + 1))
	```

	`twist_bonus` table

	Block | T | S/Z | L/J | I
	--- | --- | --- | --- | ---
	mini | 1.5 | 1.5 | 1.5 | 1.5
	regular | 2.5 | 2.5 | 2.5 | N/A

## Event-driven(instead of realtime)

* Zero delay, operations are applied as fast as possible.

* No gravity, infinite hold swap at any time

* Pending Attack won't apply until

	* a drop without line clear will drain the pending garbages

	* max 5 attacks pending, or early attacks will pop

* All blocks generated at y=22(bottom = 0)

* Die, if after board change shadow block is totally invisible

	There are two types of board changes: hard drop and garbage overflow.
