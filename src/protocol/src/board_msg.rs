use serde::{Deserialize, Serialize};

use crate::KeyType;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum BoardMsg {
	Attacked(u32),
	KeyEvent(KeyType),
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum BoardReply {
	Ok(u32),
	Die,
	GarbageOverflow,
}
