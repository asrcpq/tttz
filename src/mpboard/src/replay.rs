extern crate tttz_protocol;
use tttz_protocol::BoardMsg;

use std::time::{SystemTime, UNIX_EPOCH};

pub struct Replay {
	start_time: SystemTime,
	data: Vec<(u128, BoardMsg)>,
	block_seq: Vec<u8>,
}

impl Default for Replay {
	fn default() -> Replay {
		Replay {
			start_time: SystemTime::now(),
			data: Vec::new(),
			block_seq: Vec::new(),
		}
	}
}

impl Replay {
	pub fn push_block(&mut self, code: u8) {
		self.block_seq.push(code);
	}

	pub fn push_operation(&mut self, board_msg: BoardMsg) {
		let since_the_epoch = self.start_time
			.duration_since(UNIX_EPOCH)
			.expect("Time went backwards");
		self.data.push((since_the_epoch.as_micros(), board_msg));
	}
}
