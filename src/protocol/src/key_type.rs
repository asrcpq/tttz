use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Copy)]
pub enum KeyType {
	Nothing,
	Left,
	Right,
	LLeft,
	RRight,
	SonicDrop,
	HardDrop,
	Hold,
	Rotate,
	RotateReverse,
	RotateHalf,
}
