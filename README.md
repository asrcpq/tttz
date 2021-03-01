# My Action Puzzle Game

## Require

* rust(cargo)

* a VT100 compatible terminal

## Quickstart (for debian)

1. git clone ...

2. sudo apt install cargo

3. cd to project root

4. bash quick\_play\_ai.sh

5. press r

## Rules

* No gravity(blocks can move down), infinite hold swap

* Pending Attack won't apply until a drop without line clear

* 40 lines of pending attack cause sudden death

* SRS kick table(with 180 degree kick)

* tspin and back-to-back bonus

## Gameplay

* h: move left

* l: move right

* k: hard drop

* j: soft drop

* K: move down 5

* J: move down 1

* Space: hold

* r: suicide/start matching
