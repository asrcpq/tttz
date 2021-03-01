extern crate mpboard;
use crate::client_manager::ClientManager;
use mpboard::board::Board;

use std::collections::HashSet;
use std::net::UdpSocket;

extern crate bincode;

pub struct Client {
	pub id: i32,
	pub dc_ids: HashSet<i32>,
	// 1: waiting
	// 2: in-game
	// 3: pairing
	pub state: i32,
	pub board: Board,
	pub attack_target: i32,
	pub display_update: bool,
}

impl Client {
	pub fn new(id: i32) -> Client {
		Client {
			id,
			dc_ids: HashSet::new(),
			state: 1,
			board: Board::new(id),
			attack_target: 0,
			display_update: true,
		}
	}

	pub fn init_board(&mut self) {
		self.board = Board::new(self.id);
	}

	pub fn send_display(
		&mut self,
		socket: &UdpSocket,
		client_manager: &ClientManager,
	) {
		let msg = bincode::serialize(&self.board.display).unwrap();
		let mut new_dc_ids: HashSet<i32> = HashSet::new();
		for dc_id in self.dc_ids.drain() {
			let dc_addr =
				if let Some(addr) = client_manager.get_addr_by_id(dc_id) {
					addr
				} else {
					eprintln!(
						"A removed client: {} was viewing {}",
						dc_id, self.id
					);
					continue;
				};
			socket.send_to(&msg, dc_addr).unwrap();
			new_dc_ids.insert(dc_id);
		}
		self.dc_ids = new_dc_ids;
	}

	// die = false
	fn process_key(&mut self, words: &[&str]) -> bool {
		if words.len() == 1 {
			self.board.hold();
		} else {
			match words[1] {
				"r" => return false,
				"h" => {
					self.board.move1(1);
				}
				"H" => {
					self.board.move2(1);
				}
				"l" => {
					self.board.move1(-1);
				}
				"L" => {
					self.board.move2(-1);
				}
				"k" => {
					self.board.press_up();
				}
				"j" => {
					self.board.press_down();
				}
				"J" => {
					self.board.slowdown(1);
				}
				"K" => {
					self.board.slowdown(5);
				}
				"z" => {
					self.board.rotate(-1);
				}
				"x" => {
					self.board.rotate(1);
				}
				"d" => {
					self.board.rotate(2);
				}
				ch => {
					eprintln!("Unknown key {}", ch);
				}
			}
		}
		if !self.board.calc_shadow() {
			return false;
		}
		true
	}

	// die = true
	pub fn handle_msg(&mut self, words: &[&str]) -> bool {
		if self.state != 2 {
			self.display_update = false;
			return false;
		}
		if words[0] == "key" {
			self.display_update = true;
			if !self.process_key(words) {
				return true;
			}
		} else if words[0] == "attack" {
			let id = match words[1].parse::<i32>() {
				Ok(id) => {
					if id == self.id {
						// on garbage sending, the attacked needs to be popped from clients
						// which is impossible when the attacker is already popped
						eprintln!("Self attacking is not allowed");
						self.display_update = false;
					}
					eprintln!("Attacking {}", id);
					id
				}
				Err(_) => {
					eprintln!("Invalid attack msg: attack {}", words[1]);
					self.display_update = false;
					return false;
				}
			};
			self.attack_target = id;
			self.display_update = false;
		} else {
			eprintln!("Unknown msg: {:?}", words);
			self.display_update = false;
		}
		false
	}
}
