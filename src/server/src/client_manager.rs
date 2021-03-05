extern crate bimap;
use crate::client::Client;
use crate::server::SOCKET;
use bimap::BiMap;
use std::collections::HashMap;
use std::net::SocketAddr;

pub struct ClientManager {
	id_alloc: i32,
	clients: HashMap<i32, Client>,
	pub id_addr: BiMap<i32, SocketAddr>,
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
	pub fn view_by_id(&mut self, id: i32) -> Option<&Client> {
		self.clients.get(&id)
	}

	pub fn tmp_pop_by_id(&mut self, id: i32) -> Option<Client> {
		self.clients.remove(&id)
	}

	pub fn tmp_push_by_id(&mut self, id: i32, client: Client) {
		// reject repeat push
		assert!(self.clients.insert(id, client).is_none());
	}

	pub fn new_client_by_addr(&mut self, src: SocketAddr) -> i32 {
		let mut client = Client::new(self.id_alloc, src);
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
		match self.id_addr.get_by_left(&id) {
			Some(i) => Some(*i),
			None => None,
		}
	}

	// return none if not exist
	pub fn get_id_by_addr(&self, addr: SocketAddr) -> Option<i32> {
		match self.id_addr.get_by_right(&addr) {
			Some(i) => Some(*i),
			None => None,
		}
	}

	pub fn pair_apply(&mut self, client1: &mut Client, client2: &mut Client) {
		let id1 = client1.id;
		let id2 = client2.id;
		client1.pair_success(id2);
		client2.pair_success(id1);

		let addr1 = self.get_addr_by_id(id1).unwrap();
		let addr2 = self.get_addr_by_id(id2).unwrap();
		SOCKET
			.send_to(format!("startvs {}", id2).as_bytes(), addr1)
			.unwrap();
		SOCKET
			.send_to(format!("startvs {}", id1).as_bytes(), addr2)
			.unwrap();

		client2.board.update_display();
		client2.send_display(self);
		client1.board.update_display();
		client1.send_display(self);
	}

	pub fn pair_attempt(&mut self, mut client: &mut Client) {
		if self.pending_client == client.id {
			// the pending client is just ourselves
			return;
		}
		if client.state == 3 && self.pending_client != 0 {
			// pairing succeed
			let target_id = self.pending_client;
			let another_client = self.tmp_pop_by_id(target_id);
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
