extern crate lazy_static;
extern crate bincode;
extern crate rand;
extern crate termion;
extern crate bimap;
use bimap::BiMap;

extern crate mpboard;
use mpboard::board;

use board::Board;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::net::UdpSocket;

struct Server {
	socket: UdpSocket,
	in_game: bool,
	clients: HashMap<i32, Client>,
	id_addr: BiMap<i32, SocketAddr>,
	id_alloc: i32,
}

impl Server {
	pub fn new(bind_addr: &str) -> Server {
		Server {
			socket: UdpSocket::bind(bind_addr).unwrap(),
			in_game: false,
			clients: HashMap::new(),
			id_addr: BiMap::new(),
			id_alloc: 1,
		}
	}

	fn client_connect(&mut self, src: SocketAddr) {
		let mut client = Client::new(self.id_alloc);
		client.dc_ids.push(self.id_alloc);
		self.clients.insert(self.id_alloc, client);
		eprintln!("Assign id {}", self.id_alloc);
		self.socket
			.send_to(format!("ok {}", self.id_alloc).as_bytes(), src)
			.unwrap();
		self.id_addr.insert(self.id_alloc, src);
		self.id_alloc += 1;
	}

	pub fn main_loop(&mut self) {
		let mut buf = [0; 1024];
		loop {
			let (amt, src) = self.socket.recv_from(&mut buf).unwrap();
			let matched_id = if let Some(id) = self.id_addr.get_by_right(&src) {
				*id
			} else {
				0 // should never be matched in clients
			};
			let mut client = match self.clients.remove(&matched_id) {
				Some(client) => {
					client
				}
				None => {
					if std::str::from_utf8(&buf[..amt])
						.unwrap()
						.starts_with("new client")
					{
						self.client_connect(src);
					} else {
						eprintln!("Unknown client: {:?}", src);
					}
					continue
				}
			};
			let msg = std::str::from_utf8(&buf[..amt]).unwrap();
			if msg.starts_with("quit") {
				self.id_addr.remove_by_left(&client.id).unwrap();
				continue
			} else if msg.starts_with("get clients") {
				let mut return_msg = String::new();
				for (key, _) in &self.id_addr {
					return_msg = format!("{}{} ", return_msg, key);
				}
				return_msg.pop();
				self.socket.send_to(&return_msg.as_bytes(), src).unwrap();
			} else if msg.starts_with("view ") {
				let id = std::str::from_utf8(&buf[5..amt])
					.unwrap()
					.parse::<i32>()
					.unwrap();
				if self.id_addr.contains_left(&id) {
					let mut client_to_view = self.clients.remove(&id).unwrap();
					client_to_view.dc_ids.push(id);
					self.clients.insert(id, client_to_view);
				} else {
					eprintln!("Client {} try to view nonexist {}", client.id, id);
				}
			} else {
				if client.handle_msg(&mut buf[..amt]) {
					client.board.update_display();
					if client.board.attack_pool > 0 {
						if let Some(addr) = self.id_addr.get_by_left(&client.attack_target) {
							eprintln!("{} attack {} with {}",
								matched_id,
								client.attack_target,
								client.board.attack_pool,
							);
							client.board.pending_attack += client.board.attack_pool;
							self.socket.send_to(
								format!("sigatk {}", client.board.attack_pool).as_bytes(),
								addr,
							).unwrap();
						} else {
							eprintln!("Client {} is attacking nonexistent target {}",
								client.id,
								client.attack_target,
							);
							eprintln!("{:?}", self.id_addr);
						}
						client.board.attack_pool = 0;
					}
					let msg = bincode::serialize(&client.board.display).unwrap();
					let mut new_dc_ids: Vec<i32> = Vec::new();
					for dc_id in client.dc_ids.drain(..) {
						// The gc of dc_addr happens here, so the check is necessary,
						// since the sender is temporarily removed from hashmap
						// we will perform an extra check for it
						let dc_addr = if let Some(addr) = self.id_addr.get_by_left(&dc_id) {
							addr
						} else if matched_id == dc_id {
							&src
						} else {
							continue
						};
						self.socket
							.send_to(&msg, dc_addr)
							.unwrap();
						new_dc_ids.push(dc_id);
					}
					client.dc_ids = new_dc_ids;
				}
			}
			self.clients.insert(matched_id, client);
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

	// return: whether to update and send display to this client
	pub fn handle_msg(&mut self, msg: &[u8]) -> bool {
		let str_msg = std::str::from_utf8(msg).unwrap();
		if str_msg.starts_with("key ") {
			if self.state == 1 {
				if msg[4] == b'r' {
					self.board = Board::new(self.id);
					self.state = 2;
					return true
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
					eprintln!("Game over: id {}", self.id);
					self.state = 1;
					return false
				}
				return true;
			}
		} else if str_msg.starts_with("attack ") {
			let id = match str_msg[7..].parse::<i32>() {
				Ok(id) => {
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
}

fn main() {
	let mut server = Server::new("127.0.0.1:23124");
	server.main_loop();
}
