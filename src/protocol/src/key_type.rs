use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum KeyType {
	Left,
	Right,
	LLeft,
	RRight,
	Down1,
	Down5,
	SoftDrop,
	HardDrop,
	Hold,
	Rotate,
	RotateReverse,
	RotateFlip,
}
