use crate::board::Board;
use tttz_ruleset::*;

// clone is used when revert rotation test
#[derive(Clone, Debug)]
pub struct Block {
	pub code: u8,
	pub pos: (i32, i32),
	pub rotation: i8,
}

impl Block {
	pub fn rotate(&mut self, dr: i8) {
		let old_rot = self.rotation;
		self.rotation += dr;
		while self.rotation < 0 {
			self.rotation += 4;
		}
		while self.rotation >= 4 {
			self.rotation -= 4;
		}
		let tmp_old = SRP[self.code as usize][old_rot as usize];
		let tmp = SRP[self.code as usize][self.rotation as usize];
		self.pos.0 -= tmp_old.0;
		self.pos.1 -= tmp_old.1;
		self.pos.0 += tmp.0;
		self.pos.1 += tmp.1;
	}

	pub fn compress(&self) -> [u8; 4] {
		let mut ret = [0u8; 4];
		ret[0] = self.pos.0 as u8;
		ret[1] = self.pos.1 as u8;
		ret[2] = self.code;
		ret[3] = self.rotation as u8;
		ret
	}

	pub fn decompress(data: &[u8]) -> Self {
		Block {
			code: data[2],
			pos: (data[0] as i32, data[1] as i32),
			rotation: data[3] as i8,
		}
	}

	pub fn new(code: u8) -> Block {
		Block {
			code,
			pos: (INITIAL_POS[code as usize], 38),
			rotation: 0,
		}
	}

	// each square pos relative to block pos
	pub fn getpos_internal(&self) -> [(u8, u8); 4] {
		BPT[self.code as usize][self.rotation as usize]
	}

	pub fn getpos(&self) -> [(u8, u8); 4] {
		let mut ret = [(0, 0); 4];
		for block_id in 0..4 {
			let tmp = BPT[self.code as usize][self.rotation as usize]
				[block_id as usize];
			let px = self.pos.0 as u8 + tmp.0;
			let py = self.pos.1 as u8 + tmp.1;
			ret[block_id as usize].0 = px;
			ret[block_id as usize].1 = py;
		}
		ret
	}

	pub fn bottom_pos(&self) -> i32 {
		self.pos.1 - BLOCK_HEIGHT[self.code as usize][self.rotation as usize]
			+ 1
	}

	pub fn test(&self, board: &Board) -> bool {
		for block_id in 0..4 {
			let tmp = BPT[self.code as usize][self.rotation as usize]
				[block_id as usize];
			let px = self.pos.0 + tmp.0 as i32;
			let py = self.pos.1 + tmp.1 as i32;
			if !board.is_pos_vacant((px, py)) {
				return false;
			}
		}
		true
	}
}
