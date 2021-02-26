extern crate lazy_static;
extern crate rand;
extern crate termion;

extern crate mpboard;
use mpboard::board;

use board::Board;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::net::UdpSocket;

struct Server {
	socket: UdpSocket,
	in_game: bool,
	clients: HashMap<SocketAddr, Client>,
	id_addr: HashMap<i32, SocketAddr>,
	id_alloc: i32,
}

impl Server {
	pub fn new(bind_addr: &str) -> Server {
		Server {
			socket: UdpSocket::bind(bind_addr).unwrap(),
			in_game: false,
			clients: HashMap::new(),
			id_addr: HashMap::new(),
			id_alloc: 0,
		}
	}

	pub fn main_loop(&mut self) {
		let mut buf = [0; 1024];
		loop {
			let (amt, src) = self.socket.recv_from(&mut buf).unwrap();
			eprintln!("{}", std::str::from_utf8(&buf[..amt]).unwrap());
			let mut client = match self.clients.remove(&src) {
				Some(client) => {
					client
				}
				None => {
					if std::str::from_utf8(&buf[..amt])
						.unwrap()
						.starts_with("new dc ")
					{
						let id = std::str::from_utf8(&buf[7..amt])
							.unwrap()
							.parse::<i32>()
							.unwrap();
						println!("{:?}", self.id_addr);
						match self.id_addr.get(&id) {
							None => {
								eprintln!("A dc client ask {}, but it does not exist", id);
								self.socket.send_to(b"ko", src).unwrap();
							}
							Some(addr) => {
								self.clients.get_mut(addr).unwrap().dc_addrs.push(src);
								self.socket.send_to(b"ok", src).unwrap();
							}
						}
					} else if std::str::from_utf8(&buf[..amt])
						.unwrap()
						.starts_with("new cc")
					{
						self.clients.insert(src, Client::new(self.id_alloc));
						// TODO: proper serialization
						self.socket
							.send_to(format!("{}", self.id_alloc).as_bytes(), src)
							.unwrap();
						self.id_addr.insert(self.id_alloc, src);
						self.id_alloc += 1;
					} else {
						eprintln!("Unknown client: {:?}", src);
					}
					continue
				}
			};
			if buf[0] == b'q' {
				for dc_addr in client.dc_addrs.iter() {
					self.socket.send_to(b"quit", dc_addr).unwrap();
				}
				self.id_addr.remove(&client.id).unwrap();
			} else {
				client.handle_msg(&mut buf[..amt]);
				
				for dc_addr in client.dc_addrs.iter() {
					let msg = self.build_msg(&client);
					self.socket
						.send_to(&msg, dc_addr)
						.unwrap();
				}
				self.clients.insert(src, client);
			}
			// Do not write anything here, note the continue in match branch
		}
	}

	// This should be O(1), but currently rust cannot concat bytestring
	fn build_msg(&self, client: &Client) -> [u8; 218] {
		let mut target: [u8; 218] = [0; 218];
		// only send visible part
		target[0..200].clone_from_slice(&client.board.color[200..400]);
		target[200..208].clone_from_slice(&client.board.shadow_block.getpos()[..]);
		target[208] = client.board.shadow_block.code;
		target[209..217].clone_from_slice(&client.board.tmp_block.getpos()[..]);
		target[217] = client.board.tmp_block.code;
		target
	}
}

struct Client {
	id: i32,
	dc_addrs: Vec<SocketAddr>,
	ready: bool,
	board: Board,
}

impl Client {
	pub fn new(id: i32) -> Client {
		Client {
			id,
			dc_addrs: Vec::new(),
			ready: false,
			board: Default::default(),
		}
	}

	pub fn handle_msg(&mut self, buf: &mut [u8]) {
		if std::str::from_utf8(buf).unwrap().starts_with("key ") {
			match buf[4] as char {
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
				_ => {
					panic!();
				}
			}
			self.board.calc_shadow();
		}
	}
}

fn main() {
	let mut server = Server::new("127.0.0.1:23124");
	server.main_loop();
}
