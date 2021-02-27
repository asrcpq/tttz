extern crate lazy_static;
extern crate bincode;
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
			id_alloc: 1,
		}
	}

	fn client_connect(&mut self, src: SocketAddr) {
		let mut client = Client::new(self.id_alloc);
		client.dc_addrs.push(src);
		self.clients.insert(src, client);
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
			eprintln!("{}", std::str::from_utf8(&buf[..amt]).unwrap());
			let mut client = match self.clients.remove(&src) {
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
			if buf[0] == b'q' {
				for dc_addr in client.dc_addrs.iter() {
					self.socket.send_to(b"quit", dc_addr).unwrap();
				}
				self.id_addr.remove(&client.id).unwrap();
			} else {
				client.handle_msg(&mut buf[..amt]);
				client.board.update_display();
				let msg = bincode::serialize(&client.board.display).unwrap();
				for dc_addr in client.dc_addrs.iter() {
					self.socket
						.send_to(&msg, dc_addr)
						.unwrap();
				}
				self.clients.insert(src, client);
			}
			// Do not write anything here, note the continue in match branch
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
			board: Board::new(id),
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
				ch => {
					eprintln!("Unknown key {}", ch);
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
