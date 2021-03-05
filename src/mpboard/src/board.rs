use crate::block::Block;
use crate::display::Display;
use crate::random_generator::RandomGenerator;
use crate::srs_data::*;
use rand::Rng;
use std::collections::VecDeque;

pub struct Board {
	ontop: bool,
	pub tmp_block: Block,
	pub shadow_block: Block,
	pub rg: RandomGenerator,
	pub display: Display,
	pub attack_pool: u32,
	pub garbages: VecDeque<u32>,
	height: usize,
}

impl Board {
	pub fn new(id: i32) -> Board {
		let mut rg: RandomGenerator = Default::default();
		let mut board = Board {
			ontop: true,
			tmp_block: Block::new(rg.get()),
			shadow_block: Block::new(0),
			rg,
			display: Display::new(id),
			attack_pool: 0,
			garbages: VecDeque::new(),
			height: 40,
		};
		board.calc_shadow();
		board
	}

	fn is_pos_inside(&self, pos: (i32, i32)) -> bool {
		if pos.0 < 0 || pos.1 < 0 {
			return false;
		}
		if pos.0 >= 10 || pos.1 >= 40 {
			return false;
		}
		true
	}

	pub fn is_pos_vacant(&self, pos: (i32, i32)) -> bool {
		if !self.is_pos_inside(pos) {
			return false;
		}
		self.display.color[pos.0 as usize + pos.1 as usize * 10] == 7
	}

	fn movedown1_nohard(&mut self) -> bool {
		self.ontop = false;
		self.tmp_block.pos.1 += 1;
		if !self.tmp_block.test(self) {
			self.tmp_block.pos.1 -= 1;
			return false;
		}
		true
	}

	pub fn slowdown(&mut self, dy: u8) {
		let first_visible = 21
			- BLOCK_HEIGHT[(self.tmp_block.code * 4
				+ self.tmp_block.rotation as u8) as usize];
		if self.tmp_block.pos.1 < first_visible {
			for _ in self.tmp_block.pos.1..first_visible {
				self.movedown1_nohard();
			}
		} else {
			for _ in 0..dy {
				if !self.movedown1_nohard() {
					break;
				}
			}
		}
	}

	pub fn move1(&mut self, dx: i32) -> bool {
		self.tmp_block.pos.0 -= dx;
		if !self.tmp_block.test(self) {
			self.tmp_block.pos.0 += dx;
			return false;
		}
		true
	}

	pub fn move2(&mut self, dx: i32) {
		while self.move1(dx) {}
	}

	pub fn rotate(&mut self, dr: i8) -> bool {
		if self.tmp_block.code == 3 {
			return false;
		}
		let revert_block = self.tmp_block.clone();
		self.tmp_block.rotate(dr);
		let std_pos = self.tmp_block.pos;
		let len = if dr == 2 { 6 } else { 5 };
		for wkid in 0..len {
			let left_offset = (dr == -1) as i8 * 40;
			let idx = (revert_block.rotation * len * 2 + left_offset + wkid * 2)
				as usize;
			let wkd: &Vec<i32> = if dr == 2 {
				&FWKD
			} else if revert_block.code == 0 {
				&IWKD
			} else {
				&WKD
			};
			self.tmp_block.pos.0 = std_pos.0 + wkd[idx];
			self.tmp_block.pos.1 = std_pos.1 + wkd[idx + 1];
			if self.ontop {
				self.tmp_block.pos.1 = 0;
			}
			if self.tmp_block.test(self) {
				return true;
			}
		}
		self.tmp_block = revert_block;
		false
	}

	pub fn hold(&mut self) {
		if self.display.hold == 7 {
			self.display.hold = self.tmp_block.code;
			self.tmp_block = Block::new(self.rg.get());
		} else {
			let tmp = self.display.hold;
			self.display.hold = self.tmp_block.code;
			self.tmp_block = Block::new(tmp);
		}
		self.ontop = true;
	}

	pub fn soft_drop(&mut self) -> bool {
		self.ontop = false;
		if self.shadow_block.pos.1 == self.tmp_block.pos.1 {
			return false;
		}
		self.tmp_block.pos.1 = self.shadow_block.pos.1;
		true
	}

	// return count of lines eliminated
	pub fn checkline(&mut self, ln: Vec<usize>) -> u32 {
		let mut elims = Vec::new();
		for each_ln in ln.iter() {
			let mut flag = true;
			for i in each_ln * 10..(each_ln + 1) * 10 {
				if self.display.color[i] == 7 {
					flag = false;
				}
			}
			if flag {
				elims.push(each_ln);
			}
		}
		if elims.is_empty() {
			self.display.combo = 0;
			return 0;
		}
		let mut movedown = 0;
		for i in (0..40).rev() {
			let mut flag = false;
			for elim in elims.iter() {
				if i == **elim {
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
			for j in 0..10 {
				self.display.color[(i + movedown) * 10 + j] =
					self.display.color[i * 10 + j];
			}
		}
		movedown as u32
	}

	// return 0: none, 1: mini, 2: regular
	fn test_tspin(&mut self) -> u32 {
		if self.tmp_block.code == 5 {
			self.tmp_block.pos.0 -= 1;
			if self.tmp_block.test(self) {
				return 0;
			}
			self.tmp_block.pos.0 += 2;
			if self.tmp_block.test(self) {
				return 0;
			}
			self.tmp_block.pos.0 -= 1;
			self.tmp_block.pos.1 -= 1;
			if self.tmp_block.test(self) {
				return 0;
			}
			self.tmp_block.pos.1 += 1;

			let offset = self.tmp_block.rotation as usize * 4;
			for i in 0..2 {
				let check_x =
					self.tmp_block.pos.0 + TSPIN_MINI_CHECK[offset + i * 2];
				let check_y =
					self.tmp_block.pos.1 + TSPIN_MINI_CHECK[offset + i * 2 + 1];
				if self.display.color[(check_x + check_y * 10) as usize] == 7 {
					return 1;
				}
			}
			return 2;
		}
		0
	}

	// push a new attack into pending garbage queue
	pub fn push_garbage(&mut self, atk: u32) {
		if atk == 0 {
			return;
		}
		self.display.pending_attack += atk;
		self.garbages.push_back(atk);
	}

	// pull all pending garbages and write to board color
	pub fn generate_garbage(&mut self) {
		const SAME_LINE: f32 = 0.6;
		for mut count in self.garbages.drain(..) {
			let mut slot = self.rg.rng.gen_range(0..10);
			if count == 0 {
				eprintln!("Bug: zero in garbage");
				continue;
			}
			if count > 40 {
				count = 40;
			}
			for y in 0..(40 - count as usize) {
				for x in 0..10 {
					self.display.color[y * 10 + x] =
						self.display.color[(y + count as usize) * 10 + x];
				}
			}
			for y in 0..(count as usize) {
				let same = self.rg.rng.gen::<f32>();
				if same >= SAME_LINE {
					slot = self.rg.rng.gen_range(0..10);
				}
				let yy = 39 - y;
				for x in 0..10 {
					self.display.color[yy * 10 + x] = 2;
				}
				self.display.color[yy * 10 + slot] = 7;
			}
		}
		self.display.pending_attack = 0;
	}

	// should only called when attack_pool > 0
	// return true if attack is larger
	pub fn counter_attack(&mut self) -> bool {
		loop {
			// return if attack remains
			if self.garbages.is_empty() {
				break self.attack_pool > 0;
			}
			if self.garbages[0] >= self.attack_pool {
				self.garbages[0] -= self.attack_pool;
				if self.garbages[0] == 0 {
					self.garbages.pop_front();
				}
				self.display.pending_attack -= self.attack_pool;
				self.attack_pool = 0;
				break false;
			}
			let popped_lines = self.garbages.pop_front().unwrap();
			self.attack_pool -= popped_lines;
			self.display.pending_attack -= popped_lines;
		}
	}

	pub fn hard_drop(&mut self) {
		let tmppos = self.tmp_block.getpos();
		let mut lines_tocheck = Vec::new();
		// check tspin before setting color
		let tspin = self.test_tspin();
		if tspin > 0 {
			eprintln!("{} just did a {}-tspin", self.display.id, tspin);
		}
		for i in 0..4 {
			let px = tmppos[i * 2] as usize;
			let py = tmppos[i * 2 + 1] as usize;
			if py < self.height {
				self.height = py;
			}

			let mut flag = true;
			for l in lines_tocheck.iter() {
				if *l == py {
					flag = false;
				}
			}
			if flag {
				lines_tocheck.push(py);
			}

			self.display.color[px + py * 10] = self.tmp_block.code;
		}
		let line_count = self.checkline(lines_tocheck);

		// put attack amount into pool
		if line_count > 0 {
			self.height += line_count as usize;
			if self.attack_pool != 0 {
				eprintln!("Error! attack_pool not cleared.");
			}
			let offset =
				21 * (line_count - 1) as usize + self.display.combo as usize;
			if self.display.b2b {
				if tspin == 2 {
					self.attack_pool = ATK_B2B_TSPIN_REGULAR[offset];
				} else if tspin == 1 {
					self.attack_pool = ATK_B2B_TSPIN_MINI[offset];
				} else if tspin == 0 {
					if line_count == 4 {
						self.attack_pool =
							ATK_B2B_QUAD[self.display.combo as usize];
					} else {
						self.attack_pool = ATK_NORMAL[offset];
						self.display.b2b = false;
					}
				} else {
					unreachable!();
				}
			} else if tspin == 2 {
				self.attack_pool = ATK_TSPIN_REGULAR[offset];
				self.display.b2b = true;
			} else if tspin == 1 {
				self.attack_pool = ATK_TSPIN_MINI[offset];
				self.display.b2b = true;
			} else if tspin == 0 {
				if line_count == 4 {
					self.display.b2b = true;
				}
				self.attack_pool = ATK_NORMAL[offset];
			} else {
				unreachable!();
			}
			if tspin == 2 || line_count == 4 {
				self.display.b2b = true;
			}
			self.display.combo += 1;
			if self.display.combo > 20 {
				self.display.combo = 20;
			}
			if self.height == 40 {
				self.attack_pool += ATK_AC[self.display.combo as usize];
			}
		} else {
			// plain drop: attack execution
			self.generate_garbage();
			self.display.pending_attack = 0;
		}

		// new block
		self.ontop = true;
		self.tmp_block = Block::new(self.rg.get());
	}

	pub fn press_down(&mut self) {
		if !self.soft_drop() {
			self.hard_drop();
		}
	}

	pub fn press_up(&mut self) {
		self.soft_drop();
		self.hard_drop();
	}

	pub fn calc_shadow(&mut self) -> bool {
		self.shadow_block = self.tmp_block.clone();
		loop {
			self.shadow_block.pos.1 += 1;
			if !self.shadow_block.test(self) {
				if self.shadow_block.pos.1
					+ BLOCK_HEIGHT[self.tmp_block.code as usize * 4
						+ self.tmp_block.rotation as usize]
					< 21
				{
					return false;
				} else {
					self.shadow_block.pos.1 -= 1;
					return true;
				}
			}
		}
	}

	pub fn update_display(&mut self) {
		self.display.shadow_pos = self.shadow_block.getpos();
		self.display.shadow_code = self.shadow_block.code;
		self.display.tmp_pos = self.tmp_block.getpos();
		self.display.tmp_code = self.tmp_block.code;
		for i in 0..6 {
			self.display.bag_preview[i] = self.rg.bag[i];
		}
	}
}
