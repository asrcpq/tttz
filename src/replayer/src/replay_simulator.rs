use tttz_protocol::{Display, IdType};
use tttz_mpboard::{Board, Replay};
use crate::replay_counter::ReplayCounter;

pub struct ReplaySimulator {
	id: IdType,
	board: Board,
	replay: Replay,
	idx: usize, // for seeking
	// rc should reset after rewind
	pub rc: ReplayCounter,
}

pub enum SeekResult {
	Ok(Option<Display>),
	End,
}

impl ReplaySimulator {
	pub fn load(id: i32, path: &str) -> ReplaySimulator {
		let content = std::fs::read(path).unwrap();
		let replay: Replay = bincode::deserialize(&content).unwrap();
		let mut board: Board = Default::default();
		tttz_mpboard::utils::oracle(&mut board, 7, &replay.block_seq);
		assert!(!replay.block_seq.is_empty());
		tttz_mpboard::utils::oracle_garbage(
			&mut board,
			&replay.garbage_shift_check,
			&replay.garbage_slots,
		);
		ReplaySimulator {
			id,
			board,
			replay,
			idx: 0,
			rc: Default::default(),
		}
	}

	// (replies, replay end flag)
	pub fn seek_forward(&mut self, t: u128) -> SeekResult {
		let mut ret = None;
		loop {
			if self.idx == self.replay.data.len() {
				break SeekResult::End
			}
			if t < self.replay.data[self.idx].0 {
				return SeekResult::Ok(ret)
			}
			let br = self.board.handle_msg(self.replay.data[self.idx].1.clone());
			self.rc.count(&br, t);
			ret = Some(self.board.generate_display(self.id, br));
			self.idx += 1;
		}
	}

	pub fn print_rc(&self) {
		println!("{}", self.rc);
	}
}
