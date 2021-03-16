use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum GameType {
	// true = initiator
	Strategy(u64, bool),
	Speed,
	Single,
}
