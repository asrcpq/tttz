extern crate mpboard;
use mpboard::board::Board;
use crate::client_manager::ClientManager;

use std::net::SocketAddr;
use std::net::UdpSocket;

extern crate bincode;

pub struct Client {
	id: i32,
	addr: SocketAddr,
	pub dc_ids: Vec<i32>,
	state: i32,
	pub board: Board,
	pub attack_target: i32,
}

impl Client {
	pub fn new(id: i32, addr: SocketAddr,) -> Client {
		Client {
			id,
			addr,
			dc_ids: Vec::new(),
			state: 1,
			board: Board::new(id),
			attack_target: 0,
		}
	}

	pub fn send_display(&mut self, socket: &UdpSocket, client_manager: &ClientManager)
		-> Vec<i32> {
		let msg = bincode::serialize(&self.board.display).unwrap();
		let mut new_dc_ids: Vec<i32> = Vec::new();
		for dc_id in self.dc_ids.drain(..) {
			let dc_addr = if let Some(addr) = client_manager.get_addr_by_id(dc_id) {
				addr
			} else {
				eprintln!("A removed client: {} was viewing {}", dc_id, self.id);
				continue
			};
			socket.send_to(&msg, dc_addr).unwrap();
			new_dc_ids.push(dc_id);
		}
		new_dc_ids
	}

	// return: whether to update and send display to this client
	pub fn handle_msg(&mut self, msg: &[u8]) -> bool {
		let str_msg = std::str::from_utf8(msg).unwrap();
		if str_msg.starts_with("key ") {
			if msg[4] == b'r' {
				if self.state == 1 {
					self.board = Board::new(self.id);
					self.state = 2;
					return true
				} else if self.state == 2 {
					self.state = 1;
					return false
				}
			} else if self.state == 2 { // in game
				match msg[4] as char {
					'h' => {
						self.board.move1(1);
					}
					'H' => {
						self.board.move2(1);
					}
					'l' => {
						self.board.move1(-1);
					}
					'L' => {
						self.board.move2(-1);
					}
					'k' => {
						self.board.press_up();
					}
					'j' => {
						self.board.press_down();
					}
					'J' => {
						self.board.slowdown(1);
					}
					'K' => {
						self.board.slowdown(5);
					}
					'z' => {
						self.board.rotate(-1);
					}
					'x' => {
						self.board.rotate(1);
					}
					'd' => {
						self.board.rotate(2);
					}
					' ' => {
						self.board.hold();
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
			}
		} else if str_msg.starts_with("attack ") {
			let id = match str_msg[7..].parse::<i32>() {
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
					eprintln!("Invalid attack msg: {}", str_msg);
					return false;
				},
			};
			self.attack_target = id;
			return false
		}
		eprintln!("Unknown msg: {}", str_msg);
		false
	}

	fn die(&mut self) {
		eprintln!("Game over: id {}", self.id);
		self.state = 1;
	}
}
