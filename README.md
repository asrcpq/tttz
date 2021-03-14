# TTTZ

## Require

* rust(cargo)

* a modern terminal(unicode + ansi escape + 256 color)

## Features

* focus on 1v1 network combat(ai use the same interface as human)

* responsive and minimalist. zero-gravity, event-blocking.

* heavily modified rules to encourage fancy attack patterns,
making stupid AI harder to win.

See doc/rule.md for more

## Default Keybind

* h/l: move left/right

* H/L: move left/right until fail

* k: hard drop(hard drop + lock)

* j: sonic drop(hard drop without lock)

* Space: hold

* r: suicide/restart

* /: enter cmd mode

## Commands:

* `spawnai {commands}`, subcommands list:

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

* \<Enter\> or \<EOF\> exit cmd mode

* `quit`
