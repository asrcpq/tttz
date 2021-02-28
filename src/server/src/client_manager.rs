extern crate bimap;
use crate::client::Client;
use bimap::BiMap;
use std::collections::HashMap;
use std::net::SocketAddr;

pub struct ClientManager {
	id_alloc: i32,
	clients: HashMap<i32, Client>,
	pub id_addr: BiMap<i32, SocketAddr>,
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
	pub fn tmp_pop_by_id(&mut self, id: i32) -> Option<Client> {
		self.clients.remove(&id)
	}

	pub fn tmp_push_by_id(&mut self, id: i32, client: Client) {
		// reject repeat push
		assert!(self.clients.insert(id, client).is_none());
	}

	pub fn new_client_by_addr(&mut self, src: SocketAddr) -> i32 {
		let mut client = Client::new(self.id_alloc);
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
}
