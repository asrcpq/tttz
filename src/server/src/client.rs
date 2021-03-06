extern crate tttz_mpboard;
extern crate tttz_protocol;
use tttz_protocol::{KeyType, ServerMsg};
use crate::client_manager::ClientManager;
use crate::server::SOCKET;
use tttz_mpboard::board::Board;
use std::borrow::Cow;
use std::collections::HashSet;
use std::net::SocketAddr;

pub struct Client {
	pub id: i32,
	addr: SocketAddr,
	pub dc_ids: HashSet<i32>,
	// 1: waiting
	// 2: in-game
	// 3: pairing
	pub state: i32,
	pub board: Board,
	pub attack_target: i32,
	pub display_update: bool,
}

impl Client {
	pub fn new(id: i32, addr: SocketAddr) -> Client {
		Client {
			id,
			addr,
			dc_ids: HashSet::new(),
			state: 1,
			board: Board::new(id),
			attack_target: 0,
			display_update: true,
		}
	}

	pub fn init_board(&mut self) {
		self.board = Board::new(self.id);
	}

	pub fn pair_success(&mut self, target_id: i32) {
		self.state = 2;
		self.dc_ids.insert(target_id);
		self.attack_target = target_id;
		self.init_board();
		self.send_msg(ServerMsg::Start(target_id));
	}

	pub fn send_msg(&self, msg: ServerMsg) {
		SOCKET.send_to(&msg.serialized(), self.addr).unwrap();
	}

	pub fn broadcast_msg(
		&self,
		client_manager: &ClientManager,
		msg: &ServerMsg,
	) { // TODO maintain the view list in client, when exit do cleaning
		for dc_id in self.dc_ids.iter() {
			let dc_addr =
				if let Some(addr) = client_manager.get_addr_by_id(*dc_id) {
					addr
				} else {
					eprintln!(
						"A removed client: {} was viewing {}",
						dc_id, self.id
					);
					continue;
				};
			SOCKET.send_to(&msg.serialized(), dc_addr).unwrap();
		}
	}

	pub fn send_display(&mut self, client_manager: &ClientManager) {
		self.broadcast_msg(client_manager, &ServerMsg::Display(Cow::Borrowed(&self.board.display)));
	}

	// true = die
	pub fn process_key(&mut self, key_type: KeyType) -> bool {
		if self.state != 2 { // not playing
			return false;
		}
		match key_type {
			KeyType::Hold => {
				self.board.hold();
			}
			KeyType::Left => {
				self.board.move1(1);
			}
			KeyType::LLeft => {
				self.board.move2(1);
			}
			KeyType::Right => {
				self.board.move1(-1);
			}
			KeyType::RRight => {
				self.board.move2(-1);
			}
			KeyType::HardDrop => {
				if self.board.press_up() {
					return true;
				}
			}
			KeyType::SoftDrop => {
				if self.board.press_down() {
					return true;
				}
			}
			KeyType::Down1 => {
				self.board.slowdown(1);
			}
			KeyType::Down5 => {
				self.board.slowdown(5);
			}
			KeyType::RotateReverse => {
				self.board.rotate(-1);
			}
			KeyType::Rotate => {
				self.board.rotate(1);
			}
			KeyType::RotateFlip => {
				self.board.rotate(2);
			}
		}
		// return value ignored, only board change cause death
		self.board.calc_shadow();
		false
	}

	// true = death
	pub fn flush_garbage(&mut self, max: usize) -> bool {
		let mut flag = false;
		self.board.generate_garbage(max);
		if !self.board.calc_shadow() {
			eprintln!("SERVER: garbage pop shadow death");
			flag = true;
		}
		if self.board.height < 0 {
			eprintln!("SERVER: Height overflow death {}", self.board.height);
			flag = true;
		}
		self.board.update_display();
		flag
	}
}
