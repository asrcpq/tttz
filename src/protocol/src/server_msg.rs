use serde::{Deserialize, Serialize};

use crate::Display;
use crate::IdType;
use crate::SoundEffect;
use crate::MsgEncoding;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum ServerMsg {
	// response of new client
	AllocId(IdType),
	ClientList(Vec<IdType>),
	// receiver, amount
	Attack(IdType, u32),
	// opponent, or 0 in single player mode
	Start(IdType),
	// sender
	Request(IdType),
	// ask someone to request a match to id
	Invite(IdType),
	// true = win
	GameOver(bool),
	// or kicked
	Terminate,
	Display(Display),
	SoundEffect(IdType, SoundEffect),
}

impl ServerMsg {
	pub fn from_serialized(
		buf: &[u8],
	) -> Result<ServerMsg, Box<bincode::ErrorKind>> {
		bincode::deserialize(buf)
	}

	pub fn serialized(&self, me: MsgEncoding) -> Vec<u8> {
		match me {
			MsgEncoding::Bincode => bincode::serialize(self).unwrap(),
			MsgEncoding::Json => serde_json::to_string(self).unwrap().as_bytes().to_vec(),
		}
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
