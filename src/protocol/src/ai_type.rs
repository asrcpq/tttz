extern crate serde;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum AiType {
	Strategy,
	Speed(u64), // sleep
}
