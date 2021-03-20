use bimap::BiMap;

use crate::client::{Client, ClientState};
use tttz_protocol::{IdType, MsgEncoding, ServerMsg};

use std::collections::HashMap;
use std::net::SocketAddr;

pub struct ClientManager {
	id_alloc: IdType,
	clients: HashMap<IdType, Client>,
	id_addr: BiMap<IdType, SocketAddr>,
}

impl Default for ClientManager {
	fn default() -> ClientManager {
		ClientManager {
			id_alloc: 1,
			clients: HashMap::new(),
			id_addr: BiMap::new(),
		}
	}
}

impl ClientManager {
	pub fn check_id(&self, id: IdType) -> bool {
		self.id_addr.contains_left(&id)
	}

	pub fn get_attack_target(&self, id: IdType) -> IdType {
		self.view_by_id(id).unwrap().attack_target
	}

	pub fn in_match(&self, id: IdType) -> bool {
		self.view_by_id(id).unwrap().state == ClientState::InMatch
	}

	pub fn is_idle(&self, id: IdType) -> bool {
		match self.view_by_id(id) {
			Some(client) => client.state == ClientState::Idle,
			None => false,
		}
	}

	pub fn is_pairing(&self, id: IdType) -> bool {
		self.view_by_id(id).unwrap().state == ClientState::Pairing
	}

	pub fn set_state(&mut self, id: IdType, state: ClientState) {
		self.view_mut_by_id(id).unwrap().state = state;
	}

	pub fn view_by_id(&self, id: IdType) -> Option<&Client> {
		self.clients.get(&id)
	}

	pub fn view_mut_by_id(&mut self, id: IdType) -> Option<&mut Client> {
		self.clients.get_mut(&id)
	}

	pub fn clients(&self) -> impl Iterator<Item = IdType> + '_ {
		self.id_addr.iter().map(|(&x, _)| x)
	}

	pub fn broadcast<'a>(
		&self,
		receivers: impl Iterator<Item = &'a IdType>,
		msg: &ServerMsg,
	) {
		for &receiver in receivers {
			self.send_msg_by_id(receiver, msg);
		}
	}

	pub fn send_msg_by_id(&self, id: IdType, msg: &ServerMsg) {
		if let Some(client) = self.view_by_id(id) {
			client.send_msg(msg);
		} else {
			eprintln!("Send msg failed");
		}
	}

	pub fn new_client_by_addr(
		&mut self,
		src: SocketAddr,
		met: MsgEncoding,
		client_type: &str,
	) -> IdType {
		let client = Client::new(self.id_alloc, src, met, client_type);
		self.clients.insert(self.id_alloc, client);
		eprintln!("Assign id {}", self.id_alloc);
		self.id_addr.insert(self.id_alloc, src);
		self.id_alloc += 1;
		self.id_alloc - 1
	}

	pub fn pop_by_id(&mut self, id: IdType) -> Option<Client> {
		self.id_addr.remove_by_left(&id).unwrap();
		self.clients.remove(&id)
	}

	// return none if not exist
	// pub fn get_addr_by_id(&self, id: IdType) -> Option<SocketAddr> {
	// 	self.id_addr.get_by_left(&id).copied()
	// }

	// return none if not exist
	pub fn get_id_by_addr(&self, addr: SocketAddr) -> Option<IdType> {
		self.id_addr.get_by_right(&addr).copied()
	}

	pub fn pair_apply(&mut self, id1: IdType, id2: IdType) {
		self.view_mut_by_id(id1).unwrap().pair_success(id2);
		if id2 == 0 {
			return;
		} // single player
		self.view_mut_by_id(id2).unwrap().pair_success(id1);
	}
}
