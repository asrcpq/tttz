use tttz_ruleset::*;
use tttz_protocol::Piece;
use std::ops::{Deref, Index, IndexMut};

type Colors = Vec<[u8; 10]>;

pub struct Field {
	pub color: Colors,
}

impl Default for Field {
	fn default() -> Field {
		Field {
			color: vec![[b' '; 10]; 40],
		}
	}
}

impl Deref for Field {
	type Target = Colors;

	fn deref(&self) -> &Self::Target {
		&self.color
	}
}

impl Index<usize> for Field {
	type Output = [u8; 10];
	fn index(&self, index: usize) -> &Self::Output {
		&self.color[index]
	}
}

impl IndexMut<usize> for Field {
	fn index_mut(&mut self, index: usize) -> &mut [u8; 10] {
		&mut self.color[index]
	}
}

impl Field {
	pub fn from_color(color: &[[u8; 10]]) -> Self {
		Field {
			color: color.to_vec(),
		}
	}
	
	pub fn test(&self, block: &Piece) -> bool {
		for block_id in 0..4 {
			let tmp = BPT[block.code as usize][block.rotation as usize]
				[block_id as usize];
			let px = block.pos.0 + tmp.0;
			let py = block.pos.1 + tmp.1;
			if !self.is_pos_vacant((px, py)) {
				return false;
			}
		}
		true
	}

	fn test_twist2(&self, block: &mut Piece) -> bool {
		block.pos.0 -= 1;
		if self.test(&block) {
			block.pos.0 += 1;
			return false;
		}
		block.pos.0 += 2;
		if self.test(&block) {
			block.pos.0 -= 1;
			return false;
		}
		block.pos.0 -= 1;
		block.pos.1 += 1;
		if self.test(&block) {
			block.pos.1 -= 1;
			return false;
		}
		block.pos.1 -= 1;
		true
	}

	// return 0: fail, 1: normal, 2: twist(both)
	pub fn rotate(&self, block: &mut Piece, dr: i8) -> u32 {
		let code = block.code;
		let rotation = block.rotation;
		if code == 3 {
			return 0;
		}
		block.rotation = (rotation + dr).rem_euclid(4);
		let std_pos = block.pos;
		for wkp in kick_iter(code, rotation, dr) {
			block.pos.0 = std_pos.0 + wkp.0 as i8;
			block.pos.1 = std_pos.1 + wkp.1 as i8;
			if self.test(block) {
				if self.test_twist(block) > 0 {
					return 2;
				} else {
					return 1;
				}
			}
		}
		0
	}

	// test all types of twists
	// return 0: none, 1: mini, 2: regular
	pub fn test_twist(&self, block: &mut Piece) -> u32 {
		// No o spin
		if block.code == 3 {
			return 0;
		}
		if !self.test_twist2(block) {
			return 0;
		}
		// No mini i spin
		if block.code == 0 {
			return 2;
		}
		let tmp = &TWIST_MINI_CHECK[block.code as usize]
			[block.rotation as usize];
		for mini_pos in tmp.iter() {
			let check_x = block.pos.0 + mini_pos.0;
			let check_y = block.pos.1 + mini_pos.1;
			if self.color[check_y as usize][check_x as usize] == b' ' {
				return 1;
			}
		}
		2
	}

	fn is_pos_inside(&self, pos: (PosType, PosType)) -> bool {
		if pos.0 < 0 || pos.1 < 0 {
			return false;
		}
		if pos.0 >= 10 || pos.1 >= self.color.len() as PosType {
			return false;
		}
		true
	}

	pub fn is_pos_vacant(&self, pos: (PosType, PosType)) -> bool {
		if !self.is_pos_inside(pos) {
			return false;
		}
		self.color[pos.1 as usize][pos.0 as usize] == b' '
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn test_is_pos_inside() {
		let field: Field = Default::default();
		assert_eq!(field.is_pos_inside((10, 40)), false);
		assert_eq!(field.is_pos_inside((10, 5)), false);
		assert_eq!(field.is_pos_inside((0, 0)), true);
		assert_eq!(field.is_pos_inside((4, 20)), true);
	}
}
