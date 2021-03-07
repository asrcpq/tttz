extern crate serde;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum SoundEffect {
	// 0: fail
	// 1: success
	// 2: twist success
	Rotate(u8),
	SoftDrop,
	Move,
	RegularDrop,
	ClearDrop,
	AttackDrop(u8), // amount
	PerfectClear,
	GarbageOverflow,
	AttackReceived,
}
