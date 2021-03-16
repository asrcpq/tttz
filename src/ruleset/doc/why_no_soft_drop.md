# Drop Implementation

TTTZ only has sonic drop and hard drop. This is because
terminals cannot detect keydown and have a slow repeat rate by default.
I want TTTZ to be completely functional on terminals without any configuration.

Some alternative ways to control a block's y position are

* "hooking" the piece to a higher place, then move it horizontally into a hole.
It is totally safe since we have infinite lock time and zero gravity.

* using 180 high kicks to lift a block.
