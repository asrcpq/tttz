# TTTZ

## Require

* rust(cargo)

* a modern terminal(unicode + ansi escape + 256 color)

## Functions

* TUI with sound, cross-platform(maybe)

* No resource file

* Replay automatically saved(working)

* Built with [cold-clear](https://github.com/MinusKelvin/cold-clear) AI

## Features

* focus on 1v1 network combat(ai use the same interface as human)

* responsive and minimalist. zero-gravity, non-realtime.

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

* `spawnai {commands}`, subcommands list:

	`algo < basic | cc >` specify algorithm: dummy bot or cold-clear

	`strategy`

	`speed [sleep_millis]`

* `kick <id>`

* `clients` list clients

* `myid`

* `free` enter single play mode

* `request <id>` send battle request to id(ai will immediately accept)

* `accept <id>` accept battle request from id

* `pair` start pairing

* \<Enter\> or \<EOF\> exit cmd mode

* `quit`
