extern crate lazy_static;
extern crate bincode;
extern crate rand;
extern crate termion;

extern crate mpboard;
use mpboard::board;

use board::Board;
use std::net::SocketAddr;
use std::net::UdpSocket;

mod client_manager;
use client_manager::ClientManager;

struct Server {
	socket: UdpSocket,
	client_manager: ClientManager,
	in_game: bool,
}

impl Server {
	pub fn new(bind_addr: &str) -> Server {
		Server {
			socket: UdpSocket::bind(bind_addr).unwrap(),
			in_game: false,
			client_manager: Default::default(),
		}
	}

	fn after_operation(&mut self, mut client: &mut Client, src: SocketAddr, matched_id: i32) {
		client.board.update_display();
		if client.board.counter_attack() {
			if let Some(addr) = self.client_manager.get_addr_by_id(client.attack_target) {
				eprintln!("{} attack {} with {}",
					matched_id,
					client.attack_target,
					client.board.attack_pool,
				);

				let mut client_target = self.client_manager
					.tmp_pop_by_id(client.attack_target)
					.unwrap();
				client_target.board.push_garbage(client.board.attack_pool);
				self.socket.send_to(
					format!("sigatk {}", client_target
						.board
						.display
						.pending_attack
					).as_bytes(),
					addr,
				).unwrap();
				self.client_manager
					.tmp_push_by_id(client.attack_target, client_target);
			} else {
				eprintln!("Client {} is attacking nonexistent target {}",
					client.id,
					client.attack_target,
				);
			}
			client.board.attack_pool = 0;
		}
		let new_dc_ids = client.send_display(&self.socket, &self.client_manager);
		client.dc_ids = new_dc_ids;
	}

	pub fn main_loop(&mut self) {
		let mut buf = [0; 1024];
		loop {
			let (amt, src) = self.socket.recv_from(&mut buf).unwrap();
			let matched_id = if let Some(id) = self.client_manager.get_id_by_addr(src) {
				id
			} else {
				0 // should never be matched in clients
			};
			let mut client = match self.client_manager.tmp_pop_by_id(matched_id) {
				Some(client) => {
					client
				}
				None => {
					if std::str::from_utf8(&buf[..amt])
						.unwrap()
						.starts_with("new client")
					{
						let new_id = self.client_manager.new_client_by_addr(src);
						self.socket
							.send_to(format!("ok {}", new_id).as_bytes(), src)
							.unwrap();
					} else {
						eprintln!("Unknown client: {:?}", src);
					}
					continue
				}
			};
			let msg = std::str::from_utf8(&buf[..amt]).unwrap();
			eprintln!("{} from {}", msg, client.id);
			if msg.starts_with("quit") {
				assert!(self.client_manager.pop_by_id(client.id).is_none());
				continue
			} else if msg.starts_with("get clients") {
				let mut return_msg = String::new();
				for (key, _) in &self.client_manager.id_addr {
					return_msg = format!("{}{} ", return_msg, key);
				}
				return_msg.pop();
				self.socket.send_to(&return_msg.as_bytes(), src).unwrap();
			} else if msg.starts_with("view ") {
				let id = std::str::from_utf8(&buf[5..amt])
					.unwrap()
					.parse::<i32>()
					.unwrap();
				if self.client_manager.id_addr.contains_left(&id) {
					eprintln!("Client {} viewing {}", client.id, id);
					let mut client_to_view = self.client_manager.tmp_pop_by_id(id).unwrap();
					client_to_view.dc_ids.push(matched_id);
					self.client_manager.tmp_push_by_id(id, client_to_view);
				} else {
					eprintln!("Client {} try to view nonexist {}", client.id, id);
				}
			} else {
				if client.handle_msg(&mut buf[..amt]) {
					// display is included in after_operation
					self.after_operation(&mut client, src, matched_id);
				}
			}
			self.client_manager.tmp_push_by_id(matched_id, client);
			// Do not write anything here, note the continue in match branch
		}
	}
}

struct Client {
	id: i32,
	dc_ids: Vec<i32>,
	state: i32,
	board: Board,
	attack_target: i32,
}

impl Client {
	pub fn new(id: i32) -> Client {
		Client {
			id,
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

fn main() {
	let mut server = Server::new("127.0.0.1:23124");
	server.main_loop();
}
