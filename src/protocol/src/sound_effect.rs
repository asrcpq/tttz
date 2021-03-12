use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug, Clone)]
pub enum SoundEffect {
	// 0: fail
	// 1: success
	// 2: twist success
	Rotate(u8),
	Hold,
	SoftDrop,
	PlainDrop,
	ClearDrop,
	AttackDrop,
	AttackDrop2,
	PerfectClear,
	GarbageOverflow,
	AttackReceived,
}
