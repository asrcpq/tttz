use once_cell::sync::Lazy;

use crate::client::ClientState;
use crate::client_manager::ClientManager;
use tttz_mpboard::Game;
use tttz_protocol::{ClientMsg, IdType, MsgEncoding, ServerMsg};

use std::collections::{HashMap, HashSet};
use std::net::{SocketAddr, UdpSocket};

type GameIdType = i32;

pub static SOCKET: Lazy<UdpSocket> = Lazy::new(|| {
	let mut addr = "0.0.0.0:23124".to_string();
	let mut iter = std::env::args();
	while let Some(cmd) = iter.next() {
		if cmd == "addr" {
			if let Some(cmd) = iter.next() {
				addr = cmd.clone();
			}
		}
	}
	UdpSocket::bind(&addr).unwrap()
});

#[derive(Default)]
pub struct Server {
	client_manager: ClientManager,
	ai_threads: Vec<std::thread::JoinHandle<()>>,
	client_in_game: HashMap<IdType, GameIdType>,
	game_map: HashMap<GameIdType, Game>,
	game_id_alloc: GameIdType,
}

// encoding, client_type
fn new_client_msg_parse(msg: &[u8]) -> Result<(MsgEncoding, String), String> {
	let split =
		String::from(std::str::from_utf8(msg).map_err(|e| e.to_string())?)
			.split_whitespace()
			.map(|x| x.to_string())
			.collect::<Vec<String>>();
	if split.is_empty() {
		return Err("Message too short".to_string());
	}
	if split[0] != "new_client" {
		return Err("Unknown command".to_string());
	}
	let mut iter = split.iter();
	let mut met = MsgEncoding::Bincode;
	let mut client_type = "regular".to_string();
	while let Some(word) = iter.next() {
		match word.as_ref() {
			"json" => met = MsgEncoding::Json,
			"client_type" => {
				if let Some(word) = iter.next() {
					client_type = word.to_string();
				}
			}
			_ => {}
		}
	}
	Ok((met, client_type))
}

impl Server {
	fn parse_message(
		&mut self,
		buf: &[u8],
		src: SocketAddr,
		amt: usize,
	) -> Option<(IdType, ClientMsg)> {
		// get or create client
		let matched_id =
			if let Some(id) = self.client_manager.get_id_by_addr(src) {
				id
			} else {
				0 // should never be matched in clients
			};
		let client = self.client_manager.view_by_id(matched_id);
		if client.is_none() {
			if let Ok((met, string)) = new_client_msg_parse(&buf[..amt]) {
				let new_id =
					self.client_manager.new_client_by_addr(src, met, &string);
				self.client_manager
					.send_msg_by_id(new_id, &ServerMsg::AllocId(new_id));
			} else {
				eprintln!("Unknown client: {:?}", src);
			}
			return None;
		};
		let client = client.unwrap();
		let client_msg = match ClientMsg::from_bytes(&buf[..amt], client.met) {
			Ok(client_msg) => client_msg,
			Err(string) => {
				eprintln!(
					"[43mSERVER[0m: Parse failed from {:?}: {}",
					src, string
				);
				return None;
			}
		};
		Some((client.id, client_msg))
	}

	fn terminate_game(&mut self, client_id: IdType, winner: IdType) {
		eprintln!("Game end, {} win", winner);
		let opponent = self.client_manager.get_attack_target(client_id);
		let game_id = self.client_in_game.get(&client_id).unwrap();
		let game = self.game_map.remove(&game_id).unwrap();
		self.client_manager
			.broadcast(game.viewers.iter(), &ServerMsg::GameOver(winner));
		self.client_in_game.remove(&opponent);
		self.client_manager.set_state(client_id, ClientState::Idle);
		if opponent != 0 {
			self.client_manager.set_state(opponent, ClientState::Idle);
		}
	}

	fn try_apply_match(&mut self, id1: IdType, id2: IdType) {
		if id2 != 0 {
			if !self.client_manager.check_id(id2) {
				eprintln!("SERVER: accept: cannot find client {}", id2);
				return;
			}
			if !self.client_manager.is_pairing(id2) {
				eprintln!("SERVER: accept: but the sender is not pairing.");
				return;
			}
		}
		self.client_manager.pair_apply(id1, id2);
		let mut viewers: HashSet<IdType> = self.client_manager
			.view_by_id(id1)
			.unwrap()
			.viewers
			.iter()
			.copied()
			.collect();
		if id2 != 0 {
			viewers.extend(
				self.client_manager
					.view_by_id(id2)
					.unwrap()
					.viewers
					.iter()
			)
		}
		let new_game = Game::new(
			id1,
			id2,
			viewers.iter(),
		);
		for i in 0..if id2 == 0 { 1 } else { 2 } {
			self.client_manager.broadcast(
				new_game.viewers.iter(),
				&ServerMsg::Display(new_game.generate_display(i, 0)),
			);
		}
		self.game_map.insert(self.game_id_alloc, new_game);
		self.client_in_game.insert(id1, self.game_id_alloc);
		if id2 != 0 {
			self.client_in_game.insert(id2, self.game_id_alloc);
		}
		self.game_id_alloc += 1;
	}

	fn handle_msg(&mut self, msg: ClientMsg, client_id: IdType) {
		eprintln!("SERVER: client {} send {:?}", client_id, msg);
		match msg {
			ClientMsg::Quit => {
				eprintln!("Client {} quit", client_id);
				if self.client_manager.in_match(client_id) {
					let op = self.client_manager.get_attack_target(client_id);
					self.terminate_game(client_id, op);
				}
				self.client_manager.pop_by_id(client_id);
			}
			ClientMsg::Suicide => {
				if self.client_manager.in_match(client_id) {
					let op = self.client_manager.get_attack_target(client_id);
					self.terminate_game(client_id, op);
				}
			}
			ClientMsg::GetClients => {
				let list = self
					.client_manager
					.clients()
					.filter(|&x| x != client_id)
					.collect();
				self.client_manager
					.send_msg_by_id(client_id, &ServerMsg::ClientList(list));
			}
			ClientMsg::Kick(id) => {
				let mut flag = true;
				if id != client_id {
					// use quit, instead of kick yourself
					if self.client_manager.pop_by_id(id).is_some() {
						self.client_manager
							.send_msg_by_id(client_id, &ServerMsg::Terminate);
						flag = false;
					}
				}
				if flag {
					eprintln!("SERVER: kick failed.");
				}
			}
			ClientMsg::View(id) => {
				if let Some(client) = self.client_manager.view_mut_by_id(id) {
					client.viewers.insert(client_id);
				}
			}
			ClientMsg::NoView(id) => {
				if let Some(client) = self.client_manager.view_mut_by_id(id) {
					client.viewers.remove(&client_id);
				}
			}
			ClientMsg::SpawnAi(algo, game_type, sleep) => {
				match tttz_ai::spawn_ai(&algo, game_type, sleep) {
					Ok(join_handle) => self.ai_threads.push(join_handle),
					Err(e) => eprintln!("SERVER: {}", e),
				}
			}
			ClientMsg::Invite(id1, id2) => {
				if let Some(opponent) = self.client_manager.view_by_id(id1) {
					if opponent.state == ClientState::Idle {
						opponent.send_msg(&ServerMsg::Invite(id2));
					} else {
						eprintln!(
							"SERVER: invite: invalid invited state {:?}",
							opponent.state
						);
					}
				} else {
					eprintln!("SERVER: invite: cannot find client {}", id1);
				}
			}
			ClientMsg::Request(id) => {
				if self.client_manager.check_id(id) {
					if self.client_manager.is_idle(id) {
						self.client_manager
							.set_state(client_id, ClientState::Pairing);
						self.client_manager
							.send_msg_by_id(id, &ServerMsg::Request(client_id));
					}
				} else {
					eprintln!("SERVER: request: cannot find client {}", id);
				}
			}
			ClientMsg::Restart => {
				if !self.client_manager.is_idle(client_id) {
					eprintln!("SERVER: restart: client is not idle");
				} else {
					let opponent = self.client_manager.get_attack_target(client_id);
					if opponent == 0 {
						self.try_apply_match(client_id, 0);
					} else if self.client_manager.is_idle(opponent) {
						self.client_manager
							.set_state(client_id, ClientState::Pairing);
						self.client_manager.send_msg_by_id(
							opponent,
							&ServerMsg::Request(client_id),
						);
					}
				}
			}
			ClientMsg::Accept(id) => {
				self.try_apply_match(client_id, id);
			}
			ClientMsg::PlaySingle => {
				self.try_apply_match(client_id, 0);
			}
			ClientMsg::KeyEvent(seq, key_type) => {
				if self.client_manager.in_match(client_id) {
					let game_id = self.client_in_game.get(&client_id).unwrap();
					let game = self.game_map.get_mut(&game_id).unwrap();
					let ret = game.process_key(client_id, seq, key_type);
					for display in ret.1.into_iter() {
						self.client_manager.broadcast(
							game.viewers.iter(),
							&ServerMsg::Display(display),
						);
					}
					if ret.0 != 0 {
						// ended
						self.terminate_game(client_id, ret.0);
					}
				} else {
					eprintln!("Client send operation out of battle!");
				}
			}
		}
	}

	pub fn main_loop(&mut self) {
		let mut buf = [0; 1024];
		loop {
			// blocking
			let (amt, src) = SOCKET.recv_from(&mut buf).unwrap();
			let (id, msg) = match self.parse_message(&buf, src, amt) {
				None => continue,
				x => x.unwrap(),
			};
			// eprintln!("SERVER client {} send: {}", id, msg);
			self.handle_msg(msg, id);
			// Be aware of the continue above before writing anything here
		}
	}
}
