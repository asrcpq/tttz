use serde::{Deserialize, Serialize};
use tttz_ruleset::*;

// clone is used when revert rotation test
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Piece {
	pub code: CodeType,
	pub rotation: i8,
	pub pos: (PosType, PosType),
}

impl Piece {
	pub fn compress(&self) -> [u8; 4] {
		let mut ret = [0u8; 4];
		ret[0] = self.pos.0 as u8;
		ret[1] = self.pos.1 as u8;
		ret[2] = self.code as u8;
		ret[3] = self.rotation as u8;
		ret
	}

	pub fn decompress(data: &[u8]) -> Self {
		Piece {
			code: data[2] as PosType,
			pos: (data[0] as PosType, data[1] as PosType),
			rotation: data[3] as PosType,
		}
	}

	pub fn new(code: i8) -> Piece {
		Piece {
			code,
			pos: (INITIAL_POS[code as usize] as PosType, 38),
			rotation: 0,
		}
	}

	// each square pos relative to Piece pos
	pub fn getpos_internal(&self) -> [(PosType, PosType); 4] {
		BPT[self.code as usize][self.rotation as usize]
	}

	pub fn getpos(&self) -> [(PosType, PosType); 4] {
		let mut ret = [(0, 0); 4];
		for piece_id in 0..4 {
			let tmp = BPT[self.code as usize][self.rotation as usize]
				[piece_id as usize];
			let px = self.pos.0 + tmp.0;
			let py = self.pos.1 + tmp.1;
			ret[piece_id as usize].0 = px;
			ret[piece_id as usize].1 = py;
		}
		ret
	}
}
