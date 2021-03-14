# Drop Implementation

TTTZ only has sonic drop and hard drop.
This is because terminals cannot detect keydown and have a slow repeat rate by default.
I want TTTZ to be completely functional on terminals without any configuration.
(But maybe softdrop will be supported as a non-default compile feature?)

Although you cannot move a piece one block down,
you can still control its y position by

* "hooking" the piece to a higher place, then move it horizontally into a hole.
It is totally safe since we have infinite lock time and zero gravity.

* using 180 high kicks to lift a block.
