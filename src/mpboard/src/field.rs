use std::ops::{Deref, Index, IndexMut};
use tttz_protocol::Piece;
use tttz_ruleset::*;

use std::collections::HashSet;

type Colors = Vec<[u8; 10]>;

#[derive(Clone)]
pub struct Field {
	pub color: Colors,
	pub height: i32,
}

use std::fmt;
impl fmt::Debug for Field {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		for row in self.iter().rev() {
			for &ch in row.iter() {
				write!(f, "{} ", if ch == b' ' { '0' } else { ch as char })?;
			}
			writeln!(f)?;
		}
		Ok(())
	}
}

impl Default for Field {
	fn default() -> Field {
		Field {
			color: vec![[b' '; 10]; 40],
			height: 0,
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
	pub fn get_heights(color: &Vec<[u8; 10]>) -> [PosType; 10] {
		let mut heights: [PosType; 10] = [0; 10];
		'outer: for i in 0..10 {
			let mut j: usize = color.len() - 1;
			loop {
				if color[j][i] != b' ' {
					heights[i as usize] = j as PosType + 1;
					continue 'outer;
				}
				if j == 0 {
					continue 'outer;
				}
				j -= 1;
			}
		}
		heights
	}

	pub fn from_color(color: &Vec<[u8; 10]>) -> Self {
		let heights = Self::get_heights(color);
		let height = heights.iter().max().unwrap();
		Field {
			color: color.to_vec(),
			height: *height as i32,
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
		// No regular i spin
		if block.code == 0 {
			return 1;
		}
		let tmp =
			&TWIST_MINI_CHECK[block.code as usize][block.rotation as usize];
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

	// return count of lines eliminated
	pub fn checkline(&self, ln: HashSet<usize>) -> Vec<usize> {
		let mut elims = Vec::new();
		for &each_ln in ln.iter() {
			let mut flag = true;
			for x in 0..10 {
				if self[each_ln][x] == b' ' {
					flag = false;
					break
				}
			}
			if flag {
				elims.push(each_ln);
			}
		}
		elims
	}

	// set field, update height
	// return lines to check
	fn drop_set_color(&mut self, block: &Piece) -> HashSet<usize> {
		let tmppos = block.getpos();
		let mut lines_tocheck = HashSet::new();
		for each_square in tmppos.iter() {
			let px = each_square.0 as usize;
			let py = each_square.1 as usize;
			// tmp is higher, update height
			if py + 1 > self.height as usize {
				self.height = py as i32 + 1;
			}

			// generate lines that changed
			lines_tocheck.insert(py);
			self[py][px] = ID_TO_CHAR[block.code as usize];
		}
		lines_tocheck
	}

	fn proc_elim(&mut self, elims: Vec<usize>) {
		let mut movedown = 0;
		let len = self.color.len();
		for i in 0..len {
			let mut flag = false;
			for &elim in elims.iter() {
				if i == elim {
					flag = true;
					break;
				}
			}
			if flag {
				movedown += 1;
				continue;
			}
			if movedown == 0 {
				continue;
			}
			self[i - movedown] = self[i];
		}
		for i in 1..=movedown {
			self[len - i] = [b' '; 10];
		}
	}

	// linecount
	pub fn settle_block(&mut self, block: &Piece) -> u32 {
		let tocheck = self.drop_set_color(block);
		let toelim = self.checkline(tocheck);
		let ret = toelim.len();
		self.height -= ret as i32;
		self.proc_elim(toelim);
		ret as u32
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
