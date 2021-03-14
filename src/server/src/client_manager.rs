use bimap::BiMap;

use crate::client::{Client, ClientState};
use tttz_protocol::{ServerMsg, MsgEncoding, BoardReply};

use std::collections::HashMap;
use std::net::SocketAddr;

pub struct ClientManager {
	id_alloc: i32,
	clients: HashMap<i32, Client>,
	id_addr: BiMap<i32, SocketAddr>,
	pending_client: i32,
}

impl Default for ClientManager {
	fn default() -> ClientManager {
		ClientManager {
			id_alloc: 1,
			clients: HashMap::new(),
			id_addr: BiMap::new(),
			pending_client: 0,
		}
	}
}

impl ClientManager {
	pub fn view_by_id(&self, id: i32) -> Option<&Client> {
		self.clients.get(&id)
	}

	pub fn clients(&self) -> impl Iterator<Item = i32> + '_ {
		self.id_addr.iter().map(|(&x, _)| x)
	}

	pub fn send_msg_by_id(&self, id: i32, msg: &ServerMsg) {
		self.view_by_id(id).unwrap().send_msg(msg);
	}

	pub fn tmp_pop_by_id(&mut self, id: i32) -> Option<Client> {
		self.clients.remove(&id)
	}

	pub fn tmp_push_by_id(&mut self, id: i32, client: Client) {
		// reject repeat push
		assert!(self.clients.insert(id, client).is_none());
	}

	pub fn new_client_by_addr(
		&mut self,
		src: SocketAddr,
		met: MsgEncoding,
	) -> i32 {
		let mut client = Client::new(self.id_alloc, src, met);
		client.dc_ids.insert(self.id_alloc);
		self.clients.insert(self.id_alloc, client);
		eprintln!("Assign id {}", self.id_alloc);
		self.id_addr.insert(self.id_alloc, src);
		self.id_alloc += 1;
		self.id_alloc - 1
	}

	// ignore client nonexistence(but force addr map existence)
	// as for game over pop, client is already tmp-popped
	pub fn pop_by_id(&mut self, id: i32) -> Option<Client> {
		self.id_addr.remove_by_left(&id).unwrap();
		self.tmp_pop_by_id(id)
	}

	// return none if not exist
	pub fn get_addr_by_id(&self, id: i32) -> Option<SocketAddr> {
		self.id_addr.get_by_left(&id).copied()
	}

	// return none if not exist
	pub fn get_id_by_addr(&self, addr: SocketAddr) -> Option<i32> {
		self.id_addr.get_by_right(&addr).copied()
	}

	pub fn pair_apply(&mut self, client1: &mut Client, client2: &mut Client) {
		let id1 = client1.id;
		let id2 = client2.id;
		client1.pair_success(id2);
		client2.pair_success(id1);
		client2.send_display(self, client2.board.generate_display(BoardReply::Ok));
		client1.send_display(self, client1.board.generate_display(BoardReply::Ok));
	}

	pub fn pair_attempt(&mut self, mut client: &mut Client) {
		if self.pending_client == client.id {
			// the pending client is just ourselves
			return;
		}
		if client.state == ClientState::Pairing && self.pending_client != 0 {
			// pairing succeed
			let target_id = self.pending_client;
			let another_client = self.tmp_pop_by_id(target_id);
			match another_client {
				None => {}
				Some(mut pending_client) => {
					eprintln!(
						"{}:{:?} vs {}:{:?}",
						target_id,
						pending_client.state,
						client.id,
						client.state,
					);
					if pending_client.state == ClientState::Pairing {
						self.pending_client = 0;
						self.pair_apply(&mut client, &mut pending_client);
						self.tmp_push_by_id(target_id, pending_client);
						return;
					}
				}
			}
		}
		self.pending_client = client.id;
	}
}
