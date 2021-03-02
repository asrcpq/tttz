extern crate lazy_static;
extern crate mypuzzle_ai;
use mypuzzle_ai::ai1;
use crate::client::Client;
use crate::client_manager::ClientManager;
use std::net::SocketAddr;
use std::net::UdpSocket;

lazy_static::lazy_static! {
	pub static ref SOCKET: UdpSocket = UdpSocket::bind("0.0.0.0:23124").unwrap();
}

#[derive(Default)]
pub struct Server {
	client_manager: ClientManager,
	ai_threads: Vec<std::thread::JoinHandle<()>>,
}

impl Server {
	// true: kill
	fn send_attack(&mut self, id: i32, addr: SocketAddr, lines: u32) -> bool {
		// target id and attr
		let mut client_target = self.client_manager.tmp_pop_by_id(id).unwrap();
		let mut flag = false;
		client_target.board.push_garbage(lines);
		SOCKET
			.send_to(
				format!(
					"sigatk {}",
					client_target.board.display.pending_attack
				)
				.as_bytes(),
				addr,
			)
			.unwrap();
		if client_target.board.display.pending_attack > 40 {
			client_target.board.generate_garbage();
			flag = true;
		}
		self.client_manager.tmp_push_by_id(id, client_target);
		flag
	}

	fn post_operation(&mut self, mut client: &mut Client) {
		// note the size effect of counter_attack
		if client.board.attack_pool > 0 && client.board.counter_attack() {
			if let Some(addr) =
				self.client_manager.get_addr_by_id(client.attack_target)
			{
				eprintln!(
					"{} attack {} with {}",
					client.id, client.attack_target, client.board.attack_pool,
				);
				if self.send_attack(
					client.attack_target,
					addr,
					client.board.attack_pool,
				) {
					self.die(client, false);
				};
			} else {
				eprintln!(
					"Client {} is attacking nonexistent target {}",
					client.id, client.attack_target,
				);
			}
			client.board.attack_pool = 0;
		}
		client.board.update_display();
		client.send_display(&self.client_manager);
	}

	fn fetch_message(&mut self) -> Option<(Client, String, SocketAddr)> {
		// get or create client
		let mut buf = [0; 1024];
		let (amt, src) = SOCKET.recv_from(&mut buf).unwrap();
		let matched_id =
			if let Some(id) = self.client_manager.get_id_by_addr(src) {
				id
			} else {
				0 // should never be matched in clients
			};
		let client = match self.client_manager.tmp_pop_by_id(matched_id) {
			Some(client) => client,
			None => {
				if std::str::from_utf8(&buf[..amt])
					.unwrap()
					.starts_with("new client")
				{
					let new_id = self.client_manager.new_client_by_addr(src);
					SOCKET
						.send_to(format!("ok {}", new_id).as_bytes(), src)
						.unwrap();
				} else {
					eprintln!("Unknown client: {:?}", src);
				}
				return None;
			}
		};
		Some((
			client,
			String::from(std::str::from_utf8(&buf[..amt]).unwrap()),
			src,
		))
	}

	pub fn die(&mut self, mut client: &mut Client, die: bool) {
		client.state = 1;
		eprintln!("SERVER client {} gameover", client.id);
		if die {
			client.send_msg(b"die");
		} else {
			client.send_msg(b"win");
		}

		if client.attack_target == 0 {
			return;
		}
		// calc win by attack target works only in pair match mode
		if let Some(addr) =
			self.client_manager.get_addr_by_id(client.attack_target)
		{
			if die {
				SOCKET.send_to(b"win", addr).unwrap();
			} else {
				SOCKET.send_to(b"die", addr).unwrap();
			}
			let mut opponent = self
				.client_manager
				.tmp_pop_by_id(client.attack_target)
				.unwrap();
			opponent.state = 1;
			opponent.dc_ids.remove(&client.id);
			self.client_manager
				.tmp_push_by_id(client.attack_target, opponent);
		} // or the opponent has gone

		// attack_target is used before
		client.dc_ids.remove(&client.attack_target);
	}

	fn set_view(&mut self, from_id: i32, to_id: i32) {
		let viewed_client = self.client_manager.tmp_pop_by_id(to_id);
		match viewed_client {
			Some(mut viewed_client) => {
				eprintln!("Client {} viewing {}", from_id, to_id);
				viewed_client.dc_ids.insert(from_id);
				self.client_manager.tmp_push_by_id(to_id, viewed_client);
			}
			None => {
				eprintln!("Client {} try to view nonexist {}", from_id, to_id);
			}
		}
	}

	fn send_clients(&mut self, recipient_addr: SocketAddr) {
		let mut return_msg = String::new();
		for (key, _) in &self.client_manager.id_addr {
			return_msg = format!("{}{} ", return_msg, key);
		}
		return_msg.pop();
		SOCKET
			.send_to(&return_msg.as_bytes(), recipient_addr)
			.unwrap();
	}

	pub fn main_loop(&mut self) {
		loop {
			let (mut client, msg, src) = match self.fetch_message() {
				None => continue,
				x => x.unwrap(),
			};
			eprintln!("SERVER client {} send: {}", client.id, msg);
			let words = msg.split_whitespace().collect::<Vec<&str>>();
			if words[0] == "quit" {
				eprintln!("Client {} quit", client.id);
				self.die(&mut client, true);
				assert!(self.client_manager.pop_by_id(client.id).is_none());
				continue;
			} else if words[0] == "suicide" {
				self.die(&mut client, true);
			} else if words[0] == "clients" {
				self.send_clients(src);
			} else if words[0] == "view" {
				let id = words[1].parse::<i32>().unwrap_or(0);
				self.set_view(client.id, id);
			} else if words[0] == "aispawn" {
				let strategy: bool = words.get(1) == Some(&"strategy");
				let sleep = match words.get(2) {
					Some(t) => t.parse::<u64>().unwrap_or(240),
					None => {
						if strategy {
							0
						} else {
							240
						}
					},
				};
				self.ai_threads.push(std::thread::spawn(move || {
					ai1::main("127.0.0.1:23124", sleep, strategy);
				}));
			} else if words[0] == "request" {
				if let Ok(id) = words.get(1).unwrap_or(&"0").parse::<i32>() {
					if let Some(opponent) = self.client_manager.view_by_id(id) {
						if opponent.state == 1 {
							opponent.send_msg(format!("request {}", client.id).as_bytes());
						} else {
							eprintln!("SERVER: request: invalid opponent state {}", opponent.state);
						}
					} else {
						eprintln!("SERVER: request: cannot find client {}", id);
					}
				}
			} else if words[0] == "restart" {
				if let Some(opponent) = self.client_manager.view_by_id(client.attack_target) {
					if opponent.state == 1 {
						opponent.send_msg(format!("request {}", client.id).as_bytes());
					} else {
						eprintln!("SERVER: request: invalid opponent state {}", opponent.state);
					}
				}
			} else if words[0] == "accept" {
				if let Ok(id) = words[1].parse::<i32>() {
					if let Some(mut opponent) = self.client_manager.tmp_pop_by_id(id) {
						self.client_manager.pair_apply(&mut client, &mut opponent);
						self.client_manager.tmp_push_by_id(id, opponent);
					} else {
						eprintln!("SERVER: accept: cannot find client {}", id);
					}
				}
			} else if words[0] == "pair" {
				client.state = 3;
				self.client_manager.pair_attempt(&mut client);
			} else {
				// msg that may cause board refresh
				if words[0] == "free" {
					// free mode, attacking nothing
					client.init_board();
					client.state = 2;
					SOCKET.send_to(b"start", src).unwrap();
				} else if client.handle_msg(&words) {
					self.die(&mut client, true);
				}
				// update_display should always be evaluated in this cycle
				if client.display_update {
					// display is included in after_operation
					self.post_operation(&mut client);
				}
			}
			self.client_manager.tmp_push_by_id(client.id, client);
			// Do not write anything here, note the continue in match branch
		}
	}
}
