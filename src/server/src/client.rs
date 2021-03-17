use crate::server::SOCKET;
use tttz_protocol::{IdType, MsgEncoding, ServerMsg};

use std::net::SocketAddr;

#[derive(PartialEq, Clone, Debug)]
pub enum ClientState {
	Idle,
	Pairing,
	InMatch,
}

#[derive(Clone, Debug)]
pub struct Client {
	pub id: IdType,
	addr: SocketAddr,
	// 1: waiting
	// 2: in-game
	// 3: pairing
	pub state: ClientState,
	pub attack_target: IdType,
	pub met: MsgEncoding,
	pub save_replay: bool,
}

impl Client {
	pub fn new(
		id: IdType,
		addr: SocketAddr,
		met: MsgEncoding,
		client_type: &str,
	) -> Client {
		let save_replay = client_type != "train";
		Client {
			id,
			addr,
			state: ClientState::Idle,
			attack_target: 0,
			met,
			save_replay,
		}
	}

	pub fn pair_success(&mut self, target_id: IdType) {
		self.state = ClientState::InMatch;
		self.attack_target = target_id;
		self.send_msg(&ServerMsg::Start(target_id));
	}

	pub fn send_msg(&self, msg: &ServerMsg) {
		SOCKET
			.send_to(&msg.serialized(self.met), self.addr)
			.unwrap();
	}
}

impl Drop for Client {
	fn drop(&mut self) {
		self.send_msg(&ServerMsg::Terminate);
	}
}
