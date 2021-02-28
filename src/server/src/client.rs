extern crate mpboard;
use mpboard::board::Board;
use crate::client_manager::ClientManager;

use std::net::SocketAddr;
use std::net::UdpSocket;
use std::collections::HashSet;

extern crate bincode;

pub struct Client {
	pub id: i32,
	pub dc_ids: HashSet<i32>,
	// 1: waiting
	// 2: in-game
	// 3: pairing
	// 4: dying(tmp)
	pub state: i32,
	pub board: Board,
	pub attack_target: i32,
}

impl Client {
	pub fn new(id: i32) -> Client {
		Client {
			id,
			dc_ids: HashSet::new(),
			state: 1,
			board: Board::new(id),
			attack_target: 0,
		}
	}

	pub fn init_board(&mut self) {
		self.board = Board::new(self.id);
	}

	pub fn send_display(&mut self, socket: &UdpSocket, client_manager: &ClientManager) {
		let msg = bincode::serialize(&self.board.display).unwrap();
		let mut new_dc_ids: HashSet<i32> = HashSet::new();
		for dc_id in self.dc_ids.drain() {
			let dc_addr = if let Some(addr) = client_manager.get_addr_by_id(dc_id) {
				addr
			} else {
				eprintln!("A removed client: {} was viewing {}", dc_id, self.id);
				continue
			};
			socket.send_to(&msg, dc_addr).unwrap();
			new_dc_ids.insert(dc_id);
		}
		self.dc_ids = new_dc_ids;
	}

	// return: whether to update and send display to this client
	pub fn handle_msg(&mut self, msg: &str) -> bool {
		if self.state != 2 {
			return false
		}
		let mut words = msg.split_whitespace().collect::<Vec<&str>>();
		if words[0] == "key" {
			if words.len() == 1 {
				self.board.hold();
				return true
			}
			match words[1] {
				"r" => {
					self.state = 4;
					return false
				}
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
			if !self.board.calc_shadow() {
				self.die();
				return false
			}
			return true;
		} else if words[0] == "attack" {
			let id = match words[1].parse::<i32>() {
				Ok(id) => {
					if id == self.id {
						// on garbage sending, the attacked needs to be popped from clients
						// which is impossible when the attacker is already popped
						eprintln!("Self attacking is not allowed");
						return false
					}
					eprintln!("Attacking {}", id);
					id
				},
				Err(_) => {
					eprintln!("Invalid attack msg: {}", msg);
					return false;
				},
			};
			self.attack_target = id;
			return false
		}
		eprintln!("Unknown msg: {}", msg);
		false
	}

	pub fn die(&mut self) {
		eprintln!("Game over: id {}", self.id);
		self.state = 4;
	}
}
