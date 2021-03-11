# TTTZ

## Require

* rust(cargo)

* a VT100 compatible terminal

## Functions

* TUI with sound, cross-platform(maybe)

* No resource file!

* Replay automatically saved(working)

* Dummy AI, support strategy mode(moves in turn)

## Features

* focus on 1v1 combat

* can be manually played on console

* responsive(network delay is handled) and minimalist.
Zero-gravity and no timer.

* heavily modified rules to encourage fancy attack patterns,
making stupid AI harder to win.

See doc/rule.md for more

## Default Keybind

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
