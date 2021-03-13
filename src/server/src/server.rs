use crate::client::{Client, ClientState};
use crate::client_manager::ClientManager;
use tttz_ai::{BasicAi, CCBot, Thinker};
use tttz_protocol::{GameType, BoardMsg, BoardReply, ClientMsg, ServerMsg};

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
		let reply = client_target.board.handle_msg(BoardMsg::Attacked(lines));
		if let BoardReply::Ok(_) = reply {
			client_target.broadcast_msg(
				&self.client_manager,
				&ServerMsg::Attack(client_target.id, lines),
			);
		} else {
			client_target.send_display(
				&self.client_manager,
				client_target.board.generate_display(true),
			);
			if reply == BoardReply::Die {
				flag = true
			}
		}
		self.client_manager.tmp_push_by_id(id, client_target);
		flag
	}

	fn post_operation(&mut self, client: &mut Client, board_reply: &BoardReply) {
		// note the size effect of counter_attack
		if let BoardReply::Ok(atk) = *board_reply {
			if atk > 0 {
				if self
					.client_manager
					.get_addr_by_id(client.attack_target)
					.is_some()
				{
					eprintln!(
						"{} attack {} with {}",
						client.id, client.attack_target, atk,
					);
					if self.send_attack(client.attack_target, atk) {
						self.die(client, false);
					};
				} else {
					eprintln!(
						"Client {} is attacking nonexistent target {} with {}",
						client.id, client.attack_target, atk,
					);
				}
			}
		}
		let display = client.board.generate_display(*board_reply == BoardReply::GarbageOverflow);
		client.send_display(&self.client_manager, display);
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
		client.state = ClientState::Idle;
		match client.board.replay.save(&format!("{}", client.id)) {
			Ok(true) => {}
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
		if let Some(mut opponent) =
			self.client_manager.tmp_pop_by_id(client.attack_target)
		{
			self.die(&mut opponent, !die);
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
		client.state = ClientState::InMatch(GameType::Single);
		client.attack_target = 0;
		client.send_msg(ServerMsg::Start(0));
		self.post_operation(&mut client, &BoardReply::Ok(0));
	}

	fn spawn_ai(&mut self, algo: &str, game_type: GameType, sleep: u64) {
		match algo.as_ref() {
			"basic" => {
				self.ai_threads.push(std::thread::spawn(move || {
					let mut basic_ai: BasicAi = Default::default();
					basic_ai.main_loop("127.0.0.1:23124", sleep, game_type);
				}));
			}
			"basic_cover" => {
				self.ai_threads.push(std::thread::spawn(move || {
					let mut basic_ai = BasicAi {
						cover_weight: 0.5,
						hole_weight: 1.0,
						height_weight: 1.0,
					};
					basic_ai.main_loop("127.0.0.1:23124", sleep, game_type);
				}));
			}
			"cc" => {
				self.ai_threads.push(std::thread::spawn(move || {
					let mut ccbot: CCBot = Default::default();
					ccbot.main_loop("127.0.0.1:23124", sleep, game_type);
				}));
			}
			_ => {
				eprintln!("SERVER: Unknown algorithm {}", algo);
			}
		}
	}

	fn handle_msg(&mut self, msg: ClientMsg, mut client: &mut Client) -> bool {
		match msg {
			ClientMsg::Quit => {
				eprintln!("Client {} quit", client.id);
				self.die(&mut client, true);
				assert!(self.client_manager.pop_by_id(client.id).is_none());
				return false;
			}
			ClientMsg::Suicide => {
				self.die(&mut client, true);
			}
			ClientMsg::GetClients => {
				let list = self
					.client_manager
					.clients()
					.filter(|&x| x != client.id)
					.collect();
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
			ClientMsg::SpawnAi(algo, game_type, sleep) => {
				self.spawn_ai(&algo, game_type, sleep);
			}
			ClientMsg::Invite(id1, id2) => {
				if let Some(opponent) = self.client_manager.view_by_id(id1)
				{
					if opponent.state == ClientState::Idle {
						opponent.send_msg(ServerMsg::Invite(id2));
					} else {
						eprintln!(
							"SERVER: invite: invalid invited state {:?}",
							opponent.state
						);
					}
				} else {
					eprintln!("SERVER: invite: cannot find client {}", id1);
				}
			}
			ClientMsg::Request(id) => {
				if let Some(opponent) = self.client_manager.view_by_id(id) {
					if opponent.state == ClientState::Idle {
						client.state = ClientState::Pairing;
						opponent.send_msg(ServerMsg::Request(client.id));
					} else {
						eprintln!(
							"SERVER: request: invalid opponent state {:?}",
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
					if opponent.state == ClientState::Idle {
						client.state = ClientState::Pairing;
						opponent.send_msg(ServerMsg::Request(client.id));
					} else {
						eprintln!(
							"SERVER: request: invalid opponent state {:?}",
							opponent.state
						);
					}
				}
			}
			ClientMsg::Accept(id) => {
				if let Some(mut opponent) =
					self.client_manager.tmp_pop_by_id(id)
				{
					if opponent.state != ClientState::Pairing {
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
				client.state = ClientState::Pairing;
				self.client_manager.pair_attempt(&mut client);
			}
			ClientMsg::PlaySingle => {
				// terminate current game
				match client.state {
					ClientState::InMatch(_) => {},
					_ => {
						self.start_single(&mut client);
					}
				}
			}
			ClientMsg::KeyEvent(key_type) => {
				if let ClientState::InMatch(_) = client.state {
					let ret = client.process_key(key_type);
					// display is included in after_operation
					self.post_operation(&mut client, &ret);
					// update display before die
					if ret == BoardReply::Die {
						self.die(&mut client, true);
					}
				}
			}
			ClientMsg::NewClient => {
				unreachable!()
			}
		}
		true
	}

	pub fn main_loop(&mut self) {
		loop {
			let (mut client, msg) = match self.fetch_message() {
				None => continue,
				x => x.unwrap(),
			};
			eprintln!("SERVER client {} send: {}", client.id, msg);
			if self.handle_msg(msg, &mut client) {
				self.client_manager.tmp_push_by_id(client.id, client);
			}
			// Be aware of the continue above before writing anything here
		}
	}
}
