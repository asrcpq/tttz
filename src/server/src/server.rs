use crate::client::Client;
use crate::client_manager::ClientManager;
use std::net::SocketAddr;
use std::net::UdpSocket;

pub struct Server {
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
					matched_id,
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
			eprintln!("{} from {}", msg, matched_id);
			if msg.starts_with("quit") {
				assert!(self.client_manager.pop_by_id(matched_id).is_none());
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
					eprintln!("Client {} viewing {}", matched_id, id);
					let mut client_to_view = self.client_manager.tmp_pop_by_id(id).unwrap();
					client_to_view.dc_ids.push(matched_id);
					self.client_manager.tmp_push_by_id(id, client_to_view);
				} else {
					eprintln!("Client {} try to view nonexist {}", matched_id, id);
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
