use crate::client::Client;
use crate::client_manager::ClientManager;
use std::net::SocketAddr;
use std::net::UdpSocket;

pub struct Server {
	socket: UdpSocket,
	client_manager: ClientManager,
	pending_client: Option<i32>,
}

impl Server {
	pub fn new(bind_addr: &str) -> Server {
		Server {
			socket: UdpSocket::bind(bind_addr).unwrap(),
			client_manager: Default::default(),
			pending_client: None,
		}
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

				let mut client_target = self
					.client_manager
					.tmp_pop_by_id(client.attack_target)
					.unwrap();
				client_target.board.push_garbage(client.board.attack_pool);
				self.socket
					.send_to(
						format!(
							"sigatk {}",
							client_target.board.display.pending_attack
						)
						.as_bytes(),
						addr,
					)
					.unwrap();
				self.client_manager
					.tmp_push_by_id(client.attack_target, client_target);
			} else {
				eprintln!(
					"Client {} is attacking nonexistent target {}",
					client.id, client.attack_target,
				);
			}
			client.board.attack_pool = 0;
		}
		client.board.update_display();
		client.send_display(&self.socket, &self.client_manager);
	}

	fn fetch_message(&mut self) -> Option<(Client, String, SocketAddr)> {
		// get or create client
		let mut buf = [0; 1024];
		let (amt, src) = self.socket.recv_from(&mut buf).unwrap();
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
					self.socket
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

	pub fn pair_maker(&mut self, mut client: &mut Client) {
		eprintln!("Pair maker");
		if client.state == 3 && self.pending_client.is_some() {
			// pairing succeed
			let target_id = self.pending_client.unwrap();
			let another_client = self.client_manager.tmp_pop_by_id(target_id);
			match another_client {
				None => {}
				Some(mut pending_client) => {
					eprintln!(
						"{}:{} vs {}:{}",
						target_id,
						pending_client.state,
						client.id,
						client.state,
					);
					if pending_client.state == 3 {
						self.pending_client = None;
						client.attack_target = target_id;
						client.state = 2;
						pending_client.attack_target = client.id;
						pending_client.state = 2;
						client.dc_ids.insert(target_id);
						pending_client.dc_ids.insert(client.id);

						let addr1 = self
							.client_manager
							.get_addr_by_id(client.id)
							.unwrap();
						let addr2 = self
							.client_manager
							.get_addr_by_id(target_id)
							.unwrap();
						self.socket.send_to(format!("startvs {}", target_id).as_bytes(), addr1).unwrap();
						self.socket.send_to(format!("startvs {}", client.id).as_bytes(), addr2).unwrap();

						pending_client.board.update_display();
						pending_client
							.send_display(&self.socket, &self.client_manager);
						client.board.update_display();
						client.send_display(&self.socket, &self.client_manager);
						self.client_manager
							.tmp_push_by_id(target_id, pending_client);

						return;
					}
				}
			}
		}
		self.pending_client = Some(client.id);
	}

	pub fn die(&mut self, mut client: &mut Client, src: SocketAddr) {
		client.state = 1;
		self.socket.send_to(b"die", src).unwrap();

		// calc win by attack target works only in pair match mode
		if let Some(addr) =
			self.client_manager.get_addr_by_id(client.attack_target)
		{
			self.socket.send_to(b"win", addr).unwrap();
			let mut opponent = self
				.client_manager
				.tmp_pop_by_id(client.attack_target)
				.unwrap();
			opponent.state = 1;
			self.client_manager
				.tmp_push_by_id(client.attack_target, opponent);
		} // or the opponent has gone
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
				let mut return_msg = String::new();
				for (key, _) in &self.client_manager.id_addr {
					return_msg = format!("{}{} ", return_msg, key);
				}
				return_msg.pop();
				self.socket.send_to(&return_msg.as_bytes(), src).unwrap();
			} else if words[0] == "view" {
				let id = words[1].parse::<i32>().unwrap_or(0);
				let viewed_client = self.client_manager.tmp_pop_by_id(id);
				match viewed_client {
					Some(mut viewed_client) => {
						eprintln!("Client {} viewing {}", client.id, id);
						viewed_client.dc_ids.insert(client.id);
						self.client_manager.tmp_push_by_id(id, viewed_client);
					}
					None => {
						eprintln!(
							"Client {} try to view nonexist {}",
							client.id, id
						);
					}
				}
			} else if words[0] == "pair" {
				client.init_board();
				client.state = 3;
				if self.pending_client != Some(client.id) {
					self.pair_maker(&mut client);
				}
			} else {
				// msg that may cause board refresh
				if words[0] == "free" {
					// free mode, attacking nothing
					client.init_board();
					client.state = 2;
					self.socket.send_to(b"start", src).unwrap();
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
