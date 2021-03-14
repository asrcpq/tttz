#[derive(PartialEq, Eq, Hash)]
pub enum SoundEffect {
	// 0: fail
	// 1: success
	// 2: twist success
	Rotate(u8),
	Hold,
	SonicDrop,
	PlainDrop,
	ClearDrop,
	AttackDrop,
	AttackDrop2,
	PerfectClear,
	GarbageOverflow,
	AttackReceived,
}
