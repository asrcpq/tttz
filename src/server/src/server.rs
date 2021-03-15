use once_cell::sync::Lazy;

use crate::client::{Client, ClientState};
use crate::client_manager::ClientManager;
use tttz_protocol::{
	BoardMsg, BoardReply, ClientMsg, GameType, MsgEncoding, ServerMsg,
};

use std::net::{SocketAddr, UdpSocket};

pub static SOCKET: Lazy<UdpSocket> = Lazy::new(|| {
	let mut addr = "0.0.0.0:23124".to_string();
	let mut iter = std::env::args();
	while let Some(cmd) = iter.next() {
		if cmd == "addr" {
			if let Some(cmd) = iter.next() {
				addr = cmd.clone();
			}
		}
	}
	UdpSocket::bind(&addr).unwrap()
});

#[derive(Default)]
pub struct Server {
	client_manager: ClientManager,
	ai_threads: Vec<std::thread::JoinHandle<()>>,
}

fn new_client_msg_parse(msg: &[u8]) -> Result<MsgEncoding, String> {
	let split =
		String::from(std::str::from_utf8(msg).map_err(|e| e.to_string())?)
			.split_whitespace()
			.map(|x| x.to_string())
			.collect::<Vec<String>>();
	if split.is_empty() {
		return Err("Message too short".to_string());
	}
	if split[0] == "new_client" {
		if let Some(string) = split.get(1) {
			if string == "json" {
				return Ok(MsgEncoding::Json);
			}
		}
		Ok(MsgEncoding::Bincode)
	} else {
		Err("Unknown command".to_string())
	}
}

impl Server {
	// true: kill
	fn send_attack(&mut self, id: i32, lines: u32) -> bool {
		// target id and attr
		let mut client_target = self.client_manager.tmp_pop_by_id(id).unwrap();
		let mut flag = false;
		let reply = client_target.board.handle_msg(BoardMsg::Attacked(lines));
		if let BoardReply::Ok = reply {
			client_target.broadcast_msg(
				&self.client_manager,
				&ServerMsg::Attack(client_target.id, lines),
			);
		} else {
			client_target.send_display(
				&self.client_manager,
				client_target.board.generate_display(reply.clone()),
			);
			if reply == BoardReply::Die {
				flag = true
			}
		}
		self.client_manager.tmp_push_by_id(id, client_target);
		flag
	}

	fn post_operation(
		&mut self,
		client: &mut Client,
		board_reply: &BoardReply,
	) {
		// note the size effect of counter_attack
		if let BoardReply::ClearDrop(_line_clear, atk) = *board_reply {
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
		let display = client.board.generate_display(board_reply.clone());
		client.send_display(&self.client_manager, display);
	}

	fn parse_message(
		&mut self,
		buf: &[u8],
		src: SocketAddr,
		amt: usize,
	) -> Option<(Client, ClientMsg)> {
		// get or create client
		let matched_id =
			if let Some(id) = self.client_manager.get_id_by_addr(src) {
				id
			} else {
				0 // should never be matched in clients
			};
		let client = match self.client_manager.tmp_pop_by_id(matched_id) {
			Some(client) => client,
			None => {
				if let Ok(met) = new_client_msg_parse(&buf[..amt]) {
					let new_id =
						self.client_manager.new_client_by_addr(src, met);
					self.client_manager
						.send_msg_by_id(new_id, &ServerMsg::AllocId(new_id));
				} else {
					eprintln!("Unknown client: {:?}", src);
				}
				return None;
			}
		};
		let client_msg = match ClientMsg::from_bytes(&buf[..amt], client.met) {
			Ok(client_msg) => client_msg,
			Err(string) => {
				eprintln!(
					"[43mSERVER[0m: Parse failed from {:?}: {}",
					src, string
				);
				self.client_manager.tmp_push_by_id(client.id, client);
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
		client.send_msg(&ServerMsg::GameOver(!die));

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
				let ret = viewed_client.dc_ids.insert(from_id);
				eprintln!("Client {} viewing {}: {}", from_id, to_id, ret);
				self.client_manager.tmp_push_by_id(to_id, viewed_client);
			}
			None => {
				eprintln!("Client {} try to view nonexist {}", from_id, to_id);
			}
		}
	}

	fn unset_view(&mut self, from_id: i32, to_id: i32) {
		let viewed_client = self.client_manager.tmp_pop_by_id(to_id);
		match viewed_client {
			Some(mut viewed_client) => {
				let ret = viewed_client.dc_ids.remove(&from_id);
				eprintln!("Client {} unviewing {}: {}", from_id, to_id, ret);
				self.client_manager.tmp_push_by_id(to_id, viewed_client);
			}
			None => {
				eprintln!(
					"Client {} try to unview nonexist {}",
					from_id, to_id
				);
			}
		}
	}

	fn start_single(&mut self, client: &mut Client) {
		client.init_board();
		client.state = ClientState::InMatch(GameType::Single);
		client.attack_target = 0;
		client.send_msg(&ServerMsg::Start(0));
		let display = client.board.generate_display(BoardReply::Ok);
		client.send_display(&self.client_manager, display);
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
				client.send_msg(&ServerMsg::ClientList(list));
			}
			ClientMsg::Kick(id) => {
				let mut flag = true;
				if id != client.id {
					if let Some(client) = self.client_manager.pop_by_id(id) {
						client.send_msg(&ServerMsg::Terminate);
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
			ClientMsg::NoView(id) => {
				self.unset_view(client.id, id);
			}
			ClientMsg::SpawnAi(algo, game_type, sleep) => {
				match tttz_ai::spawn_ai(&algo, game_type, sleep) {
					Ok(join_handle) => self.ai_threads.push(join_handle),
					Err(e) => eprintln!("SERVER: {}", e),
				}
			}
			ClientMsg::Invite(id1, id2) => {
				if let Some(opponent) = self.client_manager.view_by_id(id1) {
					if opponent.state == ClientState::Idle {
						opponent.send_msg(&ServerMsg::Invite(id2));
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
						opponent.send_msg(&ServerMsg::Request(client.id));
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
						opponent.send_msg(&ServerMsg::Request(client.id));
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
						eprintln!(
							"SERVER: accept: but the sender is not pairing."
						);
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
					ClientState::InMatch(_) => {}
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
		}
		true
	}

	pub fn main_loop(&mut self) {
		let mut buf = [0; 1024];
		loop {
			// blocking
			let (amt, src) = SOCKET.recv_from(&mut buf).unwrap();
			let (mut client, msg) = match self.parse_message(&buf, src, amt) {
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

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn create_and_remove_client() {
		let mut server: Server = Default::default();
		let addr = "127.0.0.1:23124";
		let id = server
			.client_manager
			.new_client_by_addr(addr.parse().unwrap(), MsgEncoding::Json);
		let client = server.client_manager.tmp_pop_by_id(id).unwrap();
		server.client_manager.tmp_push_by_id(id, client);
		assert_eq!(
			server.client_manager.get_addr_by_id(id).unwrap(),
			addr.parse().unwrap()
		);
		assert!(server.client_manager.pop_by_id(id).is_some());
	}
}
