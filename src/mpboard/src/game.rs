use crate::Board;
use tttz_protocol::{BoardMsg, BoardReply, Display, IdType, KeyType};

use std::collections::HashSet;

pub struct Game {
	players: [IdType; 2],
	replies: [BoardReply; 2],
	seqs: [usize; 2],
	pub viewers: HashSet<IdType>,
	pub(in crate) boards: Vec<Board>,
}

impl Game {
	pub fn new<'a>(
		host: IdType,
		guest: IdType,
		extra_viewers: impl Iterator<Item = &'a IdType>
	) -> Game {
		let mut viewers = HashSet::new();
		viewers.insert(host);
		if guest != 0 {
			viewers.insert(guest);
		}
		for &viewer in extra_viewers {
			viewers.insert(viewer);
		}
		eprintln!("new game with viewers: {:?}", viewers);
		Game {
			players: [host, guest],
			replies: [BoardReply::Ok; 2],
			seqs: [0; 2],
			viewers,
			boards: vec![Default::default(), Default::default()],
		}
	}

	// return.0 winner id, 0 = not end
	// return.1 viewers
	// return.2 refresh info
	pub fn process_key(
		&mut self,
		cid: IdType,
		seq: usize,
		key_type: KeyType,
	) -> (i32, Vec<Display>) {
		let mut winner = 0;
		let id = if cid == self.players[0] {
			0
		} else if cid == self.players[1] {
			1
		} else {
			panic!("Process key getting invalid id");
		};
		let mut generate_list = vec![id];
		let oid = self.players[1 - id];
		self.seqs[id] = seq;
		let reply = self.boards[id].handle_msg(BoardMsg::KeyEvent(key_type));
		match reply {
			BoardReply::ClearDrop(_lc, atk) if atk > 0 && oid != 0 => {
				let recv_ret =
					self.boards[1 - id].handle_msg(BoardMsg::Attacked(atk));
				if recv_ret == BoardReply::Die {
					winner = cid;
				}
				self.replies[1 - id] = recv_ret;
				generate_list.push(1 - id);
			}
			BoardReply::Die => {
				winner = oid;
			}
			_ => {}
		}
		self.replies[id] = reply;
		let displays = generate_list
			.iter()
			.map(|&id| self.generate_display(id, self.seqs[id]))
			.collect();
		(winner, displays)
	}

	pub fn generate_display(&self, id: usize, seq: usize) -> Display {
		self.boards[id].generate_display(
			self.players[id],
			seq,
			self.replies[id],
		)
	}
}
