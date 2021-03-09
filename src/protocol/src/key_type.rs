use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum KeyType {
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
