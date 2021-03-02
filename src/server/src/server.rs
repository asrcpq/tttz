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
	fn send_attack(&mut self, id: i32, addr: SocketAddr, lines: u32) {
		// target id and attr

		let mut client_target = self.client_manager.tmp_pop_by_id(id).unwrap();
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
		self.client_manager.tmp_push_by_id(id, client_target);
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
				self.send_attack(
					client.attack_target,
					addr,
					client.board.attack_pool,
				);
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

	pub fn die(&mut self, mut client: &mut Client, src: SocketAddr) {
		client.state = 1;
		SOCKET.send_to(b"die", src).unwrap();

		if client.attack_target == 0 {
			return;
		}
		// calc win by attack target works only in pair match mode
		if let Some(addr) =
			self.client_manager.get_addr_by_id(client.attack_target)
		{
			SOCKET.send_to(b"win", addr).unwrap();
			let mut opponent = self
				.client_manager
				.tmp_pop_by_id(client.attack_target)
				.unwrap();
			opponent.state = 1;
			opponent.attack_target = 0;
			opponent.dc_ids.remove(&client.id);
			self.client_manager
				.tmp_push_by_id(client.attack_target, opponent);
		} // or the opponent has gone

		// attack_target is used before
		client.dc_ids.remove(&client.attack_target);
		client.attack_target = 0;
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
			// eprintln!("{} from {}", msg, client.id);
			let words = msg.split_whitespace().collect::<Vec<&str>>();
			if words[0] == "quit" {
				eprintln!("Client {} quit", client.id);
				self.die(&mut client, src);
				assert!(self.client_manager.pop_by_id(client.id).is_none());
				continue;
			} else if words[0] == "suicide" {
				self.die(&mut client, src);
			} else if words[0] == "get_clients" {
				self.send_clients(src);
			} else if words[0] == "view" {
				let id = words[1].parse::<i32>().unwrap_or(0);
				self.set_view(client.id, id);
			} else if words[0] == "vsai" {
				self.ai_threads.push(std::thread::spawn(|| {
					ai1::main(&[]);
				}));
			} else if words[0] == "pair" {
				client.init_board();
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
					self.die(&mut client, src);
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
