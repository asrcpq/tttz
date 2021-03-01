## Require

* rust(cargo)

* a VT100 compatible terminal

## Quickstart

* prepare rust compiler

* clone and cd into this

* run `bash quick_play_ai.sh`,
which will open a server, an user client and an ai client.

* type "pair", press enter

## Rules

* No gravity(blocks can move down), infinite hold swap

* Pending Attack won't apply until a drop without line clear

* 40 lines of pending attack cause sudden death

* SRS kick table(with 180 degree kick)

* tspin and back-to-back bonus, no pc(currently)

## Gameplay

* h/l: move left/right

* H/L: move left/right until fail

* k: hard drop

* j: soft drop

* K: move down 5

* J: move down 1

* Space: hold

* r: suicide/start matching
