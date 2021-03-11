use serde::{Deserialize, Serialize};

use crate::IdType;
use crate::Display;
use crate::SoundEffect;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum ServerMsg {
	AllocId(IdType), // response of new client
	ClientList(Vec<IdType>),
	Attack(IdType, u32), // receiver, amount
	Start(IdType), // opponent, or 0 in single player mode
	Request(IdType), // sender
	Invite(IdType), // ask someone to request a match to id
	GameOver(bool), // true = win
	Terminate, // kicked
	Display(Display), // hope this can be optimized
	SoundEffect(IdType, SoundEffect),
}

impl ServerMsg {
	pub fn from_serialized(
		buf: &[u8],
	) -> Result<ServerMsg, Box<bincode::ErrorKind>> {
		bincode::deserialize(buf)
	}

	pub fn serialized(&self) -> Vec<u8> {
		bincode::serialize(self).unwrap()
	}
}

impl std::fmt::Display for ServerMsg {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let string = match self {
			Self::AllocId(id) => format!("Ok {}", id),
			Self::Attack(id, amount) => {
				format!("sigatk {} {}", id, amount)
			}
			Self::Start(id) => {
				format!("startvs {}", id)
			}
			Self::Terminate => "kicked".to_string(),
			Self::Request(id) => {
				format!("request {}", id)
			}
			Self::Invite(id) => {
				format!("invite {}", id)
			}
			Self::ClientList(list) => {
				format!("Client list {:?}", list)
			}
			Self::GameOver(true) => "win".to_string(),
			Self::GameOver(false) => "die".to_string(),
			Self::SoundEffect(id, se) => format!("se of {}, {:?}", id, se),
			Self::Display(_) => "display".to_string(),
		};
		write!(f, "{}", string)
	}
}
