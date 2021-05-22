use serde::{Deserialize, Serialize};

use crate::KeyType;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum BoardMsg {
	Attacked(u32, u32),
	KeyEvent(KeyType),
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Copy)]
pub enum BoardReply {
	// Ok and ko are return value for softdrop, move and rotate
	// Ok is also used as return value of silent garbage queue push
	Ok,
	BadMove,
	RotateTwist,
	PlainDrop(u32),           // garbage generated
	ClearDrop(u32, u32, u32), // lineclear, atk, raw_atk
	Die,
	GarbageOverflow(u32), // garbage generated
}
