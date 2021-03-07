extern crate tttz_protocol;
use tttz_protocol::display::Display;
use tttz_protocol::{BoardMsg, BoardReply, KeyType};

use crate::block::Block;
use crate::random_generator::RandomGenerator;
use crate::srs_data::*;
use crate::replay::Replay;
use rand::Rng;

pub struct Board {
	pub tmp_block: Block,
	pub shadow_block: Block,
	pub rg: RandomGenerator,
	pub display: Display,
	pub attack_pool: u32,
	pub height: i32,
	pub replay: Replay,
}

impl Board {
	pub fn new(id: i32) -> Board {
		let replay = Default::default();
		let mut board = Board {
			tmp_block: Block::new(0), // immediately overwritten
			shadow_block: Block::new(0), // immediately overwritten
			rg: Default::default(),
			display: Display::new(id),
			attack_pool: 0,
			height: 40,
			replay,
		};
		board.spawn_block();
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

	// true = die
	pub fn handle_msg(&mut self, board_msg: BoardMsg) -> BoardReply {
		self.replay.push_operation(board_msg.clone());
		match board_msg {
			BoardMsg::KeyEvent(key_type) => match key_type {
				KeyType::Hold => {
					self.hold();
				}
				KeyType::Left => {
					self.move1(1);
				}
				KeyType::LLeft => {
					self.move2(1);
				}
				KeyType::Right => {
					self.move1(-1);
				}
				KeyType::RRight => {
					self.move2(-1);
				}
				KeyType::HardDrop => {
					if self.press_up() {
						return BoardReply::Die;
					}
				}
				KeyType::SoftDrop => {
					if self.press_down() {
						return BoardReply::Die;
					}
				}
				KeyType::Down1 => {
					self.slowdown(1);
				}
				KeyType::Down5 => {
					self.slowdown(5);
				}
				KeyType::RotateReverse => {
					self.rotate(-1);
				}
				KeyType::Rotate => {
					self.rotate(1);
				}
				KeyType::RotateFlip => {
					self.rotate(2);
				}
			},
			BoardMsg::Attacked(amount) => {
				self.push_garbage(amount);
				const MAX_GARBAGE_LEN: usize = 5;
				if self.display.garbages.len() > MAX_GARBAGE_LEN {
					if self.flush_garbage(MAX_GARBAGE_LEN) {
						return BoardReply::Die;
					} else {
						return BoardReply::GarbageOverflow;
					}
				}
			}
		}
		BoardReply::Ok
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
			if self.tmp_block.test(self) {
				return true;
			}
		}
		self.tmp_block = revert_block;
		false
	}

	fn spawn_block(&mut self) {
		let code = self.rg.get();
		self.replay.push_block(code);
		self.tmp_block = Block::new(code);
	}

	pub fn hold(&mut self) {
		if self.display.hold == 7 {
			self.display.hold = self.tmp_block.code;
			self.spawn_block();
		} else {
			let tmp = self.display.hold;
			self.display.hold = self.tmp_block.code;
			self.tmp_block = Block::new(tmp);
		}
	}

	pub fn soft_drop(&mut self) -> bool {
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

	// true = death
	pub fn flush_garbage(&mut self, max: usize) -> bool {
		let mut flag = false;
		self.generate_garbage(max);
		if !self.calc_shadow() {
			eprintln!("SERVER: garbage pop shadow death");
			flag = true;
		}
		if self.height < 0 {
			eprintln!("SERVER: Height overflow death {}", self.height);
			flag = true;
		}
		self.update_display();
		flag
	}

	// push a new attack into pending garbage queue
	pub fn push_garbage(&mut self, atk: u32) {
		if atk == 0 {
			return;
		}
		self.display.garbages.push_back(atk);
	}

	// pull all pending garbages and write to board color
	pub fn generate_garbage(&mut self, keep: usize) -> u32 {
		const SAME_LINE: f32 = 0.6;
		let mut ret = 0;
		loop {
			if self.display.garbages.len() <= keep {
				break;
			}
			let mut count = match self.display.garbages.pop_front() {
				Some(x) => x,
				None => break,
			};
			self.height -= count as i32;
			let mut slot = self.rg.rng.gen_range(0..10);
			if count == 0 {
				eprintln!("Bug: zero in garbage");
				continue;
			}
			if count > 40 {
				count = 40;
			}
			ret += count;
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
					self.display.color[yy * 10 + x] = 2; // L = white
				}
				self.display.color[yy * 10 + slot] = 7;
				if !self.tmp_block.test(self) {
					self.tmp_block.pos.1 -= 1;
				}
			}
		}
		ret
	}

	// should only called when attack_pool > 0
	// return true if attack is larger
	pub fn counter_attack(&mut self) -> bool {
		loop {
			// return if attack remains
			if self.display.garbages.is_empty() {
				break self.attack_pool > 0;
			}
			if self.display.garbages[0] >= self.attack_pool {
				self.display.garbages[0] -= self.attack_pool;
				if self.display.garbages[0] == 0 {
					self.display.garbages.pop_front();
				}
				self.attack_pool = 0;
				break false;
			}
			let popped_lines = self.display.garbages.pop_front().unwrap();
			self.attack_pool -= popped_lines;
		}
	}

	// providing whether tspin, shape offset and cleared lines
	// change self b2b and attack_pool
	fn calc_tspin_b2b(&mut self, tspin: u32, offset: usize, line_count: u32) {
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
	}

	// true: die
	pub fn hard_drop(&mut self) -> bool {
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

			// tmp is higher, update height
			if py < self.height as usize {
				self.height = py as i32;
			}

			// generate lines that changed
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
			self.height += line_count as i32;
			if self.attack_pool != 0 {
				eprintln!("Error! attack_pool not cleared.");
			}
			let offset =
				21 * (line_count - 1) as usize + self.display.combo as usize;
			self.calc_tspin_b2b(tspin, offset, line_count);
		} else {
			// plain drop: attack execution
			self.generate_garbage(0);
			if self.height < 0 {
				return true;
			}
		}

		// new block
		self.spawn_block();
		if !self.calc_shadow() {
			return true;
		}
		false
	}

	// true = death
	pub fn press_down(&mut self) -> bool {
		if !self.soft_drop() {
			return self.hard_drop();
		}
		false
	}

	// true = death
	pub fn press_up(&mut self) -> bool {
		self.soft_drop();
		self.hard_drop()
	}

	// false: die
	pub fn calc_shadow(&mut self) -> bool {
		self.shadow_block = self.tmp_block.clone();
		loop {
			self.shadow_block.pos.1 += 1;
			if !self.shadow_block.test(self) {
				self.shadow_block.pos.1 -= 1;
				if self.shadow_block.bottom_pos() < 20 {
					eprintln!(
						"SERVER: shadow_block bottom {}",
						self.shadow_block.bottom_pos()
					);
					return false;
				} else {
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

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn test_is_pos_inside() {
		let board = Board::new(1);
		assert_eq!(board.is_pos_inside((10, 40)), false);
		assert_eq!(board.is_pos_inside((10, 5)), false);
		assert_eq!(board.is_pos_inside((0, 0)), true);
		assert_eq!(board.is_pos_inside((4, 20)), true);
	}

	fn generate_solidlines(heights: [usize; 10]) -> Display {
		let mut display = Display::new(1);
		for i in 0..10 {
			for j in 0..heights[i] {
				display.color[(39 - j) * 10 + i] = 1;
			}
		}
		display
	}

	#[test]
	fn test_calc_shadow() {
		let mut board = Board::new(1);
		board.tmp_block = Block::new(1); // █▄▄
		board.display = generate_solidlines([1, 3, 2, 5, 4, 1, 2, 5, 2, 0]);
		board.calc_shadow();
		use std::collections::HashSet;
		let mut blocks: HashSet<(i32, i32)> = HashSet::new();
		blocks.insert((3, 33));
		blocks.insert((3, 34));
		blocks.insert((4, 34));
		blocks.insert((5, 34));
		let shadow_pos = board.shadow_block.getpos();
		println!("{:?} {:?}", blocks, shadow_pos);
		for i in 0..4 {
			blocks.remove(&(
				shadow_pos[i * 2] as i32,
				shadow_pos[i * 2 + 1] as i32,
			));
		}
		assert!(blocks.is_empty());
		board.move2(-1); // move to very left
		board.calc_shadow();

		blocks.insert((0, 35));
		blocks.insert((0, 36));
		blocks.insert((1, 36));
		blocks.insert((2, 36));
		let shadow_pos = board.shadow_block.getpos();
		println!("{:?} {:?}", blocks, shadow_pos);
		for i in 0..4 {
			blocks.remove(&(
				shadow_pos[i * 2] as i32,
				shadow_pos[i * 2 + 1] as i32,
			));
		}
	}
}
