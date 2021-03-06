extern crate lazy_static;
extern crate tttz_ai;
use crate::client::Client;
use crate::client_manager::ClientManager;
use tttz_ai::ai1;
use tttz_protocol::{AiType, ClientMsg, ServerMsg};
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
	fn send_attack(&mut self, id: i32, lines: u32) -> bool {
		const MAX_GARBAGE_LEN: usize = 5;
		// target id and attr
		let mut client_target = self.client_manager.tmp_pop_by_id(id).unwrap();
		let mut flag = false;
		client_target.board.push_garbage(lines);
		if client_target.board.display.garbages.len() > MAX_GARBAGE_LEN {
			flag = client_target.flush_garbage(MAX_GARBAGE_LEN);
			client_target.send_display(&self.client_manager);
		} else {
			client_target.broadcast_msg(
				&self.client_manager,
				&ServerMsg::Attack(client_target.id, lines),
			);
		}
		self.client_manager.tmp_push_by_id(id, client_target);
		flag
	}

	fn post_operation(&mut self, mut client: &mut Client) {
		// note the size effect of counter_attack
		if client.board.attack_pool > 0 && client.board.counter_attack() {
			if self
				.client_manager
				.get_addr_by_id(client.attack_target)
				.is_some()
			{
				eprintln!(
					"{} attack {} with {}",
					client.id, client.attack_target, client.board.attack_pool,
				);
				if self
					.send_attack(client.attack_target, client.board.attack_pool)
				{
					self.die(client, false);
				};
			} else {
				eprintln!(
					"Client {} is attacking nonexistent target {} with {}",
					client.id, client.attack_target, client.board.attack_pool,
				);
			}
			client.board.attack_pool = 0;
		}
		client.board.update_display();
		client.send_display(&self.client_manager);
	}

	fn fetch_message(&mut self) -> Option<(Client, ClientMsg, SocketAddr)> {
		// get or create client
		let mut buf = [0; 1024];
		let (amt, src) = SOCKET.recv_from(&mut buf).unwrap();
		let client_msg = match ClientMsg::from_serialized(&buf[..amt]) {
			Ok(client_msg) => client_msg,
			Err(_) => {
				eprintln!("[43mSERVER[0m: Parse failed from {:?}", src);
				return None
			},
		};
		let matched_id =
			if let Some(id) = self.client_manager.get_id_by_addr(src) {
				id
			} else {
				0 // should never be matched in clients
			};
		let client = match self.client_manager.tmp_pop_by_id(matched_id) {
			Some(client) => client,
			None => {
				if client_msg == ClientMsg::NewClient{
					let new_id = self.client_manager.new_client_by_addr(src);
					self.client_manager.send_msg_by_id(
						new_id,
						ServerMsg::AllocId(new_id),
					);
				} else {
					eprintln!("Unknown client: {:?}", src);
				}
				return None;
			}
		};
		Some((
			client,
			client_msg,
			src,
		))
	}

	pub fn die(&mut self, mut client: &mut Client, die: bool) {
		client.state = 1;
		eprintln!("SERVER client {} gameover", client.id);
		client.send_msg(ServerMsg::GameOver(!die));

		if client.attack_target == 0 {
			return;
		}
		// calc win by attack target works only in pair match mode
		if self.client_manager.get_addr_by_id(client.attack_target).is_some() {
			let mut opponent = self
				.client_manager
				.tmp_pop_by_id(client.attack_target)
				.unwrap();
			opponent.send_msg(ServerMsg::GameOver(die));
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
			match msg {
				ClientMsg::Quit => {
					eprintln!("Client {} quit", client.id);
					self.die(&mut client, true);
					assert!(self.client_manager.pop_by_id(client.id).is_none());
					continue;
				},
				ClientMsg::Suicide => {
					self.die(&mut client, true);
				}
				ClientMsg::GetClients => {
					self.send_clients(src);
				}
				ClientMsg::Kick(id) => {
					let mut flag = true;
					if id != client.id {
						if let Some(client) = self.client_manager.pop_by_id(id) {
							client.send_msg(ServerMsg::Terminate);
							flag = false;
						}
					}
					if flag {
						eprintln!("SERVER: kick failed.");
					}
				}
				ClientMsg::View(id) => {
					self.set_view(client.id, id);
				}
				ClientMsg::SpawnAi(ai_type) => {
					match ai_type {
						AiType::Strategy => {
							self.ai_threads.push(std::thread::spawn(move || {
								ai1::main("127.0.0.1:23124", 10, true);
							}));
						},
						AiType::Speed(sleep) => {
							self.ai_threads.push(std::thread::spawn(move || {
								ai1::main("127.0.0.1:23124", sleep, false);
							}));
						}
					} 
				}
				ClientMsg::Request(id) => {
					if let Some(opponent) = self.client_manager.view_by_id(id) {
						if opponent.state == 1 {
							client.state = 3;
							opponent.send_msg(ServerMsg::Request(client.id));
						} else {
							eprintln!(
								"SERVER: request: invalid opponent state {}",
								opponent.state
							);
						}
					} else {
						eprintln!("SERVER: request: cannot find client {}", id);
					}
				}
				ClientMsg::Restart => {
					if let Some(opponent) =
						self.client_manager.view_by_id(client.attack_target)
					{
						if opponent.state == 1 {
							client.state = 3;
							opponent.send_msg(ServerMsg::Request(client.id));
						} else {
							eprintln!(
								"SERVER: request: invalid opponent state {}",
								opponent.state
							);
						}
					}
				}
				ClientMsg::Accept(id) => {
					if let Some(mut opponent) =
						self.client_manager.tmp_pop_by_id(id)
					{
						if opponent.state != 3 {
							eprintln!("SERVER: accept: but the sender is not pairing.");
						} else {
							self.client_manager
								.pair_apply(&mut client, &mut opponent);
							self.client_manager.tmp_push_by_id(id, opponent);
						}
					} else {
						eprintln!("SERVER: accept: cannot find client {}", id);
					}
				}
				ClientMsg::Pair => {
					client.state = 3;
					self.client_manager.pair_attempt(&mut client);
				}
				ClientMsg::PlaySingle => {
					client.init_board();
					client.state = 2;
					client.attack_target = 0;
					client.send_msg(ServerMsg::Start(0));
					self.post_operation(&mut client);
				}
				ClientMsg::KeyEvent(key_type) => {
					let dieflag = client.process_key(key_type);
					// update_display should always be evaluated in this cycle
					if client.display_update {
						// display is included in after_operation
						self.post_operation(&mut client);
					}
					// update display before die
					if dieflag {
						self.die(&mut client, true);
					}
				}
				ClientMsg::NewClient => { unreachable!() }
			}
			self.client_manager.tmp_push_by_id(client.id, client);
			// Be aware of the continue above before writing anything here
		}
	}
}
