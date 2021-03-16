# TTTZ

## Require

* rust(cargo)

* a modern terminal(unicode + ansi escape + 256 color)

## Features

* Focus on 1v1 network combat(ai use the same interface as human),
optimized for high latency environment

* Responsive and minimalist. zero-gravity, event-blocking.

* Special attack rules, including all-spin bonus and twist combo multiplier
to encourage fancy attack patterns, making stupid AI harder to win.

* Replay is automatically saved. KPP, APP and PPS are computed by replayer.

## Default Keybind

* h/l: move left/right

* H/L: move left/right until fail

* k: hard drop(hard drop + lock)

* j: sonic drop(hard drop without lock)

* Space: hold

* r: give up/restart

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

* `<CR>` or `<EOF>` exit cmd mode

* `quit`
