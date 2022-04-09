use crate::replay_counter::ReplayCounter;
use tttz_mpboard::{Board, Replay};
use tttz_protocol::{Display, IdType};

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
	End(Option<Display>),
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
		if self.idx >= self.replay.data.len() {
			return SeekResult::End(None);
		}
		if t < self.replay.data[self.idx].0 {
			return SeekResult::Ok(None);
		}
		let br = self.board.handle_msg(self.replay.data[self.idx].1.clone());
		self.rc.count(&br, t);
		self.idx += 1;
		SeekResult::Ok(Some(self.board.generate_display(self.id, 0, br)))
	}

	pub fn print_rc(&self) {
		println!("{}", self.rc);
	}
}
