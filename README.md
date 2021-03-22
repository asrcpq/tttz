# TTTZ

Warning: master != work, clone tagged version instead

## Require

* rust(cargo)

* a modern terminal(unicode + ansi escape + 256 color)

## Features

* Focus on 1v1 network combat(ai use the same interface as human),
optimized for slow connections.

* Responsive and minimalist(xterm is recommended for its low input latency)

* Heavily optimized rules, including
immobile twist(and mini-twist) check, all-twist bonus, twist combo multiplier, etc.
These rules further encourage skillful attack patterns and challenge current AI implementations.

* Replay is automatically saved. KPP, APP and PPS are computed by replayer.

## Default Keybind

* h/l or Left/Right: Move

* j/k or down/up: sonic/hard drop(no soft drop)

* H/L: move until fail

* z, x, d: rotate ccw, cw, 180

* Space: hold

* r: give up/restart

* a: accept battle request

* /: enter cmd mode

## Commands:

* `spawnai <commands>*`, subcommands list:

	`algo <basic | cc | mm>` specify algorithm: dummy bot, cold-clear or MisaMino

	`strategy` ai won't move until player moves

	`speed [sleep_millis]` ai will sleep between each move

* `kick <id>`

* `clients` list clients

* `myid`

* `free` enter single play mode

* `request <id>` send battle request to id(ai will immediately accept)

* `accept <id>` accept battle request from id

* `pair` start pairing

* `<CR>` or `<EOF>` exit cmd mode

* `quit`
