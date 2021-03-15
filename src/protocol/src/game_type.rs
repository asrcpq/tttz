use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum GameType {
	Strategy(u64),
	Speed,
	Single,
}
