extern crate bincode;
extern crate serde;
use serde::{Deserialize, Serialize};

pub mod display;
use display::Display;

use std::borrow::Cow;

pub type IdType = i32;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum AiType {
	Strategy,
	Speed(u64), // sleep
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum KeyType {
	Left,
	Right,
	LLeft,
	RRight,
	Down1,
	Down5,
	SoftDrop,
	HardDrop,
	Hold,
	Rotate,
	RotateReverse,
	RotateFlip,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum ClientMsg {
	NewClient,
	Quit,
	Suicide,
	GetClients,
	PlaySingle,
	Kick(IdType),
	View(IdType),
	SpawnAi(AiType),
	Request(IdType),
	Restart,
	Accept(IdType),
	Pair,
	KeyEvent(KeyType),
}

impl ClientMsg {
	pub fn from_serialized(buf: &[u8]) -> Result<ClientMsg, Box<bincode::ErrorKind>> {
		bincode::deserialize(buf)
	}
	
	fn from_str_spawnai(words: Vec<&str>) -> Result<ClientMsg, ()> {
		if let Some(keyword) = words.get(2) {
			if keyword == &"strategy" {
				return Ok(ClientMsg::SpawnAi(AiType::Strategy))
			} else if keyword == &"speed" {
				if let Some(sleep) = words.get(3) {
					if let Ok(sleep) = sleep.parse::<u64>() {
						return Ok(ClientMsg::SpawnAi(AiType::Speed(sleep)))
					}
				}
			}
		}
		return Ok(ClientMsg::SpawnAi(AiType::Speed(240)))
	}

	pub fn from_str(input: &str) -> Result<ClientMsg, ()> {
		let split = input.split_whitespace().collect::<Vec<&str>>();
		match split[0] {
			"clients" => {
				return Ok(ClientMsg::GetClients)
			}
			"restart" => {
				return Ok(ClientMsg::Restart)
			}
			"pair" => {
				return Ok(ClientMsg::Pair)
			}
			"free" => {
				return Ok(ClientMsg::PlaySingle)
			}
			"spawnai" => {
				return Self::from_str_spawnai(split)
			}
			"request" => {
				if let Some(keyword) = split.get(1) {
					if let Ok(id) = keyword.parse::<i32>() {
						return Ok(ClientMsg::Request(id))
					}
				}
			}
			"accept" => {
				if let Some(keyword) = split.get(1) {
					if let Ok(id) = keyword.parse::<i32>() {
						return Ok(ClientMsg::Accept(id))
					}
				}
			}
			"view" => {
				if let Some(keyword) = split.get(1) {
					if let Ok(id) = keyword.parse::<i32>() {
						return Ok(ClientMsg::View(id))
					}
				}
			}
			"kick" => {
				if let Some(keyword) = split.get(1) {
					if let Ok(id) = keyword.parse::<i32>() {
						return Ok(ClientMsg::Kick(id))
					}
				}
			}
			_ => {},
		}
		return Err(())
	}

	pub fn serialized(&self) -> Vec<u8> {
		bincode::serialize(self).unwrap()
	}
}
impl std::fmt::Display for ClientMsg {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:?}", self) // just use debug
	}
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum ServerMsg<'a> {
	AllocId(IdType), // response of new client
	ClientList(Vec<i32>),
	Attack(IdType, u32), // receiver, amount
	Start(IdType), // opponent, or 0 in single player mode
	Request(IdType), // sender
	GameOver(bool), // true = win
	Terminate, // kicked
	Display(Cow<'a, Display>), // hope this can be optimized
}

impl ServerMsg<'_> {
	pub fn from_serialized<'a>(buf: &[u8]) -> Result<ServerMsg<'a>, Box<bincode::ErrorKind>> {
		bincode::deserialize(buf)
	}

	pub fn serialized(&self) -> Vec<u8> {
		bincode::serialize(self).unwrap()
	}
}

impl<'a> std::fmt::Display for ServerMsg<'a> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let string = match self {
			Self::AllocId(id) => format!("Ok {}", id),
			Self::Attack(id, amount) => {
				format!("sigatk {} {}", id, amount)
			},
			Self::Start(id) => {
				format!("startvs {}", id)
			},
			Self::Terminate => "kicked".to_string(),
			Self::Request(id) => {
				format!("request {}", id)
			},
			Self::ClientList(list) => {
				format!("Client list {:?}", list)
			}
			Self::GameOver(true) => "win".to_string(),
			Self::GameOver(false) => "die".to_string(),
			Self::Display(_) => "display".to_string(),
		};
		write!(f, "{}", string)
	}
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum BoardMsg {
	Attacked(u32),
	KeyEvent(KeyType),
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum BoardReply {
	Ok,
	Die,
	GarbageOverflow,
}
