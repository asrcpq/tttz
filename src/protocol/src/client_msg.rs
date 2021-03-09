use serde::{Deserialize, Serialize};

use crate::AiType;
use crate::KeyType;
use crate::IdType;

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
	pub fn from_serialized(
		buf: &[u8],
	) -> Result<ClientMsg, Box<bincode::ErrorKind>> {
		bincode::deserialize(buf)
	}

	fn from_str_spawnai(words: Vec<&str>) -> ClientMsg {
		if let Some(keyword) = words.get(2) {
			if keyword == &"strategy" {
				return ClientMsg::SpawnAi(AiType::Strategy);
			} else if keyword == &"speed" {
				if let Some(sleep) = words.get(3) {
					if let Ok(sleep) = sleep.parse::<u64>() {
						return ClientMsg::SpawnAi(AiType::Speed(sleep));
					}
				}
			}
		}
		ClientMsg::SpawnAi(AiType::Speed(240))
	}

	pub fn serialized(&self) -> Vec<u8> {
		bincode::serialize(self).unwrap()
	}
}

use std::str::FromStr;
impl FromStr for ClientMsg {
	type Err = ();

	fn from_str(input: &str) -> Result<Self, Self::Err> {
		let split = input.split_whitespace().collect::<Vec<&str>>();
		match split[0] {
			"clients" => return Ok(ClientMsg::GetClients),
			"restart" => return Ok(ClientMsg::Restart),
			"pair" => return Ok(ClientMsg::Pair),
			"free" => return Ok(ClientMsg::PlaySingle),
			"spawnai" => return Ok(Self::from_str_spawnai(split)),
			"request" => {
				if let Some(keyword) = split.get(1) {
					if let Ok(id) = keyword.parse::<i32>() {
						return Ok(ClientMsg::Request(id));
					}
				}
			}
			"accept" => {
				if let Some(keyword) = split.get(1) {
					if let Ok(id) = keyword.parse::<i32>() {
						return Ok(ClientMsg::Accept(id));
					}
				}
			}
			"view" => {
				if let Some(keyword) = split.get(1) {
					if let Ok(id) = keyword.parse::<i32>() {
						return Ok(ClientMsg::View(id));
					}
				}
			}
			"kick" => {
				if let Some(keyword) = split.get(1) {
					if let Ok(id) = keyword.parse::<i32>() {
						return Ok(ClientMsg::Kick(id));
					}
				}
			}
			_ => {}
		}
		Err(())
	}
}

impl std::fmt::Display for ClientMsg {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:?}", self) // just use debug
	}
}
