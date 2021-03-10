use tttz_ai::{BasicAi, SbAi, Thinker};
use tttz_protocol::{AiType, BoardMsg, BoardReply, ClientMsg, ServerMsg};
use crate::client::Client;
use crate::client_manager::ClientManager;

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
		// target id and attr
		let mut client_target = self.client_manager.tmp_pop_by_id(id).unwrap();
		let mut flag = false;
		match client_target.board.handle_msg(BoardMsg::Attacked(lines)) {
			BoardReply::Ok => {
				client_target.broadcast_msg(
					&self.client_manager,
					&ServerMsg::Attack(client_target.id, lines),
				);
			}
			BoardReply::GarbageOverflow => {
				client_target.send_display(&self.client_manager);
			}
			BoardReply::Die => {
				client_target.send_display(&self.client_manager);
				flag = true
			}
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

	fn fetch_message(&mut self) -> Option<(Client, ClientMsg)> {
		// get or create client
		let mut buf = [0; 1024];
		let (amt, src) = SOCKET.recv_from(&mut buf).unwrap();
		let client_msg = match ClientMsg::from_serialized(&buf[..amt]) {
			Ok(client_msg) => client_msg,
			Err(_) => {
				eprintln!("[43mSERVER[0m: Parse failed from {:?}", src);
				return None;
			}
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
				if client_msg == ClientMsg::NewClient {
					let new_id = self.client_manager.new_client_by_addr(src);
					self.client_manager
						.send_msg_by_id(new_id, ServerMsg::AllocId(new_id));
				} else {
					eprintln!("Unknown client: {:?}", src);
				}
				return None;
			}
		};
		Some((client, client_msg))
	}

	pub fn die(&mut self, client: &mut Client, die: bool) {
		eprintln!("SERVER: client {} gameover", client.id);
		client.state = 1;
		match client.board.replay.save(&format!("{}", client.id)) {
			Ok(true) => {},
			Ok(false) => {
				eprintln!("[32mSERVER[0m: cannot find path to write replay!");
			}
			Err(_) => {
				eprintln!("[32mSERVER[0m: write replay failed!");
			}
		}
		client.send_msg(ServerMsg::GameOver(!die));

		if client.attack_target == 0 {
			return;
		}
		// calc win by attack target works only in pair match mode
		if let Some(mut opponent) = self.client_manager
			.tmp_pop_by_id(client.attack_target) {
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

	fn start_single(&mut self, mut client: &mut Client) {
		client.init_board();
		client.state = 2;
		client.attack_target = 0;
		client.send_msg(ServerMsg::Start(0));
		self.post_operation(&mut client);
	}

	pub fn main_loop(&mut self) {
		loop {
			let (mut client, msg) = match self.fetch_message() {
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
				}
				ClientMsg::Suicide => {
					self.die(&mut client, true);
				}
				ClientMsg::GetClients => {
					let list = self
						.client_manager
						.id_addr
						.iter()
						.map(|(x, _)| *x)
						.filter(|x| *x != client.id)
						.collect();
					eprintln!("{:?} {}", list, client.id);
					client.send_msg(ServerMsg::ClientList(list));
				}
				ClientMsg::Kick(id) => {
					let mut flag = true;
					if id != client.id {
						if let Some(client) = self.client_manager.pop_by_id(id)
						{
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
				ClientMsg::SpawnAi(algo, ai_type) => {
					let (sleep, strategy) = match ai_type {
						AiType::Strategy => (10, true),
						AiType::Speed(sleep) => (sleep, false),
					};
					match algo.as_ref() {
						"sbai" => {
							self.ai_threads.push(std::thread::spawn(move || {
								let mut sbai: SbAi = Default::default();
								sbai.main_loop("127.0.0.1:23124", sleep, strategy);
							}));
						}
						"basic" => {
							self.ai_threads.push(std::thread::spawn(move || {
								let mut basic_ai = BasicAi::new();
								basic_ai.main_loop("127.0.0.1:23124", sleep, strategy);
							}));
						}
						"basic_cover" => {
							self.ai_threads.push(std::thread::spawn(move || {
								let mut basic_ai = BasicAi::new();
								basic_ai.cover_weight = 0.5;
								basic_ai.main_loop("127.0.0.1:23124", sleep, strategy);
							}));
						}
						_ => {
							eprintln!("SERVER: Unknown algorithm {}", algo);
						}
					}
				},
				ClientMsg::Invite(id1, id2) => {
					if let Some(opponent) = self.client_manager.view_by_id(id1) {
						if opponent.state == 1 {
							client.state = 3;
							opponent.send_msg(ServerMsg::Invite(id2));
						} else {
							eprintln!(
								"SERVER: invite: invalid invited state {}",
								opponent.state
							);
						}
					} else {
						eprintln!("SERVER: invite: cannot find client {}", id1);
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
					if client.attack_target == 0 {
						self.start_single(&mut client);
					} else if let Some(opponent) =
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
					// terminate current game
					if client.state == 2 {
						self.die(&mut client, true);
					}
					self.start_single(&mut client);
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
				ClientMsg::NewClient => {
					unreachable!()
				}
			}
			self.client_manager.tmp_push_by_id(client.id, client);
			// Be aware of the continue above before writing anything here
		}
	}
}
