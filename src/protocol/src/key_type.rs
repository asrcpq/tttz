use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Copy)]
pub enum KeyType {
	Nothing,
	Left,
	Right,
	LLeft,
	RRight,
	SoftDrop,
	HardDrop,
	Hold,
	Rotate,
	RotateReverse,
	RotateFlip,
}
