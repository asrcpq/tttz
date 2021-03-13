use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum GameType {
	Strategy(u64),
	Speed,
	Single,
}
