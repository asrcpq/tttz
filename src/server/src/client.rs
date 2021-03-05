extern crate mypuzzle_mpboard;
use crate::client_manager::ClientManager;
use crate::server::SOCKET;
use mypuzzle_mpboard::board::Board;
use std::collections::HashSet;
use std::net::SocketAddr;

extern crate bincode;

pub struct Client {
	pub id: i32,
	addr: SocketAddr,
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
	pub fn new(id: i32, addr: SocketAddr) -> Client {
		Client {
			id,
			addr,
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

	pub fn pair_success(&mut self, target_id: i32) {
		self.state = 2;
		self.dc_ids.insert(target_id);
		self.attack_target = target_id;
		self.init_board();
	}

	pub fn send_msg(&self, msg: &[u8]) {
		SOCKET.send_to(&msg, self.addr).unwrap();
	}

	pub fn broadcast_msg(
		&mut self,
		client_manager: &ClientManager,
		msg: &[u8],
	) {
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
			SOCKET.send_to(&msg, dc_addr).unwrap();
			new_dc_ids.insert(dc_id);
		}
		self.dc_ids = new_dc_ids;
	}

	pub fn send_display(&mut self, client_manager: &ClientManager) {
		let msg = bincode::serialize(&self.board.display).unwrap();
		self.broadcast_msg(client_manager, &msg);
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
					if self.board.press_up() {
						return false;
					}
				}
				"j" => {
					if self.board.press_down() {
						return false;
					}
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
		self.board.calc_shadow();
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
						return false;
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

	// true = death
	pub fn flush_garbage(&mut self, max: usize) -> bool {
		let mut flag = false;
		self.board.generate_garbage(max);
		self.board.calc_shadow(); // add test against move shadow block up
		if self.board.height < 0 {
			eprintln!("SERVER: Height overflow death {}", self.board.height);
			flag = true;
		}
		if self.board.tmp_block.bottom_pos() < 19 {
			// invisible + 1
			eprintln!("SERVER: invisible + 1 pop death");
			flag = true;
		}
		self.board.update_display();
		flag
	}
}
