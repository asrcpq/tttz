## Require

* rust(cargo)

* a VT100 compatible terminal

## Rules

* No gravity(blocks can move down), infinite hold swap

* Pending Attack won't apply until a drop without line clear

* 40 lines of pending attack cause sudden death

* Garbage in same attack has a 20% probability to shift

* SRS kick table(with 180 degree kick)

* tspin, b2b and pc implemented

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
