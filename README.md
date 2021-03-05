## Require

* rust(cargo)

* a VT100 compatible terminal

## Features

* SRS kick table(with 180 degree kick), with tspin, b2b and pc implemented.

* Non-realtime(described below)

* Garbage in same attack has a 30% probability to shift

## Non-realtime features

* No gravity(blocks can move down), infinite hold swap

* Pending Attack won't apply until

	* a drop without line clear will drain the pending garbages

	* early garbages will be popped to keep the length of
	garbage sequence always not greater than 5(5 attacks pending max)

* 40 lines of pending attack cause sudden death

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

* `aispawn strategy` or `aispawn speed [sleep_millis]`

* `clients` list clients

* `myid`

* `request <id>` send battle request to id(ai will immediately accept)

* `accept <id>` accept battle request from id

* `pair` start pairing

* `quit`
