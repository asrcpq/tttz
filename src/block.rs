use crate::srs_data::*;
use crate::Board;

// clone is used when revert rotation test
#[derive(Clone)]
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
		let idx_old = (self.code * 8 + old_rot as u8 * 2) as usize;
		let idx = (self.code * 8 + self.rotation as u8 * 2) as usize;
		self.pos.0 -= SRP[idx_old];
		self.pos.1 -= SRP[idx_old + 1];
		self.pos.0 += SRP[idx];
		self.pos.1 += SRP[idx + 1];
	}

	pub fn initial_pos(code: u8) -> i32 {
		match code {
			3 => 5,
			0 => 3,
			_ => 4,
		}
	}

	pub fn new(code: u8) -> Block {
		Block {
			code,
			pos: (Block::initial_pos(code), 0),
			rotation: 0,
		}
	}

	pub fn getpos(&self) -> [u16; 8] {
		let mut ret = [0u16; 8];
		for block_id in 0..4 {
			let tmp = self.code * 32 + self.rotation as u8 * 8  + block_id * 2;
			let px = self.pos.0 + BPT[tmp as usize];
			let py = self.pos.1 + BPT[tmp as usize + 1];
			ret[block_id as usize * 2] = px as u16;
			ret[block_id as usize * 2 + 1] = py as u16;
		}
		ret
	}

	pub fn test(&self, board: &Board) -> bool {
		for block_id in 0..4 {
			let tmp = self.code * 32 + self.rotation as u8 * 8  + block_id * 2;
			let px = self.pos.0 + BPT[tmp as usize];
			let py = self.pos.1 + BPT[tmp as usize + 1];
			if !board.is_pos_vacant((px, py)) {
				return false
			}
		}
		true
	}
}
