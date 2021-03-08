extern crate serde;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug, Clone)]
pub enum SoundEffect {
	Silence,
	// 0: fail
	// 1: success
	// 2: twist success
	Rotate(u8),
	SoftDrop,
	Hold,
	PlainDrop,
	ClearDrop, // combo
	AttackDrop, // amount
	PerfectClear,
	GarbageOverflow,
	AttackReceived,
}