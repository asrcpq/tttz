use tttz_protocol::BoardReply;

#[derive(PartialEq, Eq, Hash)]
pub enum SoundEffect {
	// 0: fail
	// 1: success
	// 2: twist success
	RotateTwist,
	Fail,
	SonicDrop,
	PlainDrop,
	ClearDrop,
	AttackDrop,
	AttackDrop2,
	GarbageOverflow,
	Silence,
}

impl SoundEffect {
	pub fn from_board_reply(board_reply: &BoardReply) -> SoundEffect {
		match board_reply {
			&BoardReply::ClearDrop(_lc, _atk, raw_atk) => {
				if raw_atk == 0 {
					SoundEffect::ClearDrop
				} else if raw_atk < 4 {
					SoundEffect::AttackDrop
				} else {
					SoundEffect::AttackDrop2
				}
			}
			BoardReply::RotateTwist => SoundEffect::RotateTwist,
			BoardReply::BadMove => SoundEffect::Fail,
			_ => SoundEffect::Silence,
		}
	}
}
