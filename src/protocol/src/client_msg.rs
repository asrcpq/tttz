use serde::{Deserialize, Serialize};

use crate::{GameType, IdType, KeyType};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum ClientMsg {
	Quit,
	Suicide,
	GetClients,
	PlaySingle,
	Kick(IdType),
	View(IdType),
	NoView(IdType),
	SpawnAi(String, GameType, u64), // kind, game_type, sleep
	Request(IdType),
	Invite(IdType, IdType),
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
		let mut iter = words.iter();
		let mut game_type = GameType::Speed;
		let mut sleep = 500;
		let mut algorithm = "cc".to_string();
		while let Some(&word) = iter.next() {
			match word {
				"strategy" => {
					game_type = GameType::Strategy(1000); // currently no time limit
				}
				"speed" => {
					game_type = GameType::Speed;
					if let Some(word) = iter.next() {
						if let Ok(sleep2) = word.parse::<u64>() {
							sleep = sleep2;
						}
					}
				}
				"algo" => {
					if let Some(word) = iter.next() {
						algorithm = word.to_string();
					}
				}
				_ => {}
			}
		}
		ClientMsg::SpawnAi(algorithm, game_type, sleep)
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
			"invite" => {
				if let Some(keyword) = split.get(1) {
					if let Ok(id1) = keyword.parse::<i32>() {
						if let Some(keyword) = split.get(2) {
							if let Ok(id2) = keyword.parse::<i32>() {
								return Ok(ClientMsg::Invite(id1, id2));
							}
						}
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
			"noview" => {
				if let Some(keyword) = split.get(1) {
					if let Ok(id) = keyword.parse::<i32>() {
						return Ok(ClientMsg::NoView(id));
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
			_ => {},
		}
		Err(())
	}
}

impl std::fmt::Display for ClientMsg {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:?}", self) // just use debug
	}
}
