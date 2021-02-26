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
			match self.clients.get_mut(&src) {
				Some(client) => {
					if buf[0] == b'q' {
						for dc_addr in client.dc_addrs.iter() {
							self.socket.send_to(b"quit", dc_addr).unwrap();
						}
					}
					client.handle_msg(&mut buf[..amt]);
					for dc_addr in client.dc_addrs.iter() {
						self.socket
							.send_to(&client.board.color[..], dc_addr)
							.unwrap();
					}
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
						match self.id_addr.get(&id) {
							None => {
								eprintln!("A dc client ask {}, but it does not exist", id);
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
				}
			}
		}
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
