use crate::board::Board;
use tttz_ruleset::*;

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
			pos: (INITIAL_POS[code as usize], 1),
			rotation: 0,
		}
	}

	pub fn getpos(&self) -> [u8; 8] {
		let mut ret = [0u8; 8];
		for block_id in 0..4 {
			let tmp = self.code * 32 + self.rotation as u8 * 8 + block_id * 2;
			let px = self.pos.0 + BPT[tmp as usize];
			let py = self.pos.1 + BPT[tmp as usize + 1];
			ret[block_id as usize * 2] = px as u8;
			ret[block_id as usize * 2 + 1] = py as u8;
		}
		ret
	}

	pub fn bottom_pos(&self) -> i32 {
		self.pos.1
			+ BLOCK_HEIGHT[(self.code * 4 + self.rotation as u8) as usize]
			- 1
	}

	pub fn test(&self, board: &Board) -> bool {
		for block_id in 0..4 {
			let tmp = self.code * 32 + self.rotation as u8 * 8 + block_id * 2;
			let px = self.pos.0 + BPT[tmp as usize];
			let py = self.pos.1 + BPT[tmp as usize + 1];
			if !board.is_pos_vacant((px, py)) {
				return false;
			}
		}
		true
	}
}
