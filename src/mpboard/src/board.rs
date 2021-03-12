use tttz_protocol::Display;
use tttz_protocol::{BoardMsg, BoardReply, KeyType, SoundEffect};
use tttz_ruleset::*;

use crate::block::Block;
use crate::random_generator::RandomGenerator;
use crate::replay::Replay;
use rand::Rng;

use std::collections::{HashSet, VecDeque};

pub struct Board {
	id: i32,
	pub tmp_block: Block,
	pub shadow_block: Block,
	pub rg: RandomGenerator,
	pub(in crate) color: [[u8; 10]; 40],
	hold: u8,
	cm: u32,
	tcm: u32,
	garbages: VecDeque<u32>,
	pub attack_pool: u32,
	pub last_se: SoundEffect,
	pub height: i32,
	pub replay: Replay,
}

impl Board {
	pub fn new(id: i32) -> Board {
		let replay = Default::default();
		let mut board = Board {
			id,
			tmp_block: Block::new(0),    // immediately overwritten
			shadow_block: Block::new(0), // immediately overwritten
			rg: Default::default(),
			color: [[7; 10]; 40],
			hold: 7,
			cm: 0,
			tcm: 0,
			garbages: VecDeque::new(),
			attack_pool: 0,
			last_se: SoundEffect::Silence,
			height: 0,
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
		self.color[pos.1 as usize][pos.0 as usize] == 7
	}

	// true = die
	pub fn handle_msg(&mut self, board_msg: BoardMsg) -> BoardReply {
		self.replay.push_operation(board_msg.clone());
		match board_msg {
			BoardMsg::KeyEvent(key_type) => match key_type {
				KeyType::Nothing => {}
				KeyType::Hold => {
					self.hold();
					self.last_se = SoundEffect::Hold;
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
				if self.garbages.len() > MAX_GARBAGE_LEN {
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

	fn move1(&mut self, dx: i32) -> bool {
		self.tmp_block.pos.0 -= dx;
		if !self.tmp_block.test(self) {
			self.tmp_block.pos.0 += dx;
			return false;
		}
		true
	}

	fn move2(&mut self, dx: i32) {
		while self.move1(dx) {}
	}

	fn rotate2(&mut self, dr: i8) -> u8 {
		let code = self.tmp_block.code;
		let rotation = self.tmp_block.rotation;
		if code == 3 {
			return 0;
		}
		self.tmp_block.rotate(dr);
		let std_pos = self.tmp_block.pos;
		for wkp in kick_iter(code, rotation, dr) {
			self.tmp_block.pos.0 = std_pos.0 + wkp.0 as i32;
			self.tmp_block.pos.1 = std_pos.1 + wkp.1 as i32;
			if self.tmp_block.test(self) {
				if self.test_twist() > 0 {
					return 2;
				} else {
					return 1;
				}
			}
		}
		0
	}

	// rotate2 is extracted for AI
	fn rotate(&mut self, dr: i8) {
		let revert_block = self.tmp_block.clone();
		let ret = self.rotate2(dr);
		if ret == 0 {
			self.tmp_block = revert_block;
		}
		self.last_se = SoundEffect::Rotate(ret);
	}

	fn spawn_block(&mut self) {
		let code = self.rg.get();
		self.replay.push_block(code);
		self.tmp_block = Block::new(code);
	}

	fn hold(&mut self) {
		if self.hold == 7 {
			self.hold = self.tmp_block.code;
			self.spawn_block();
		} else {
			let tmp = self.hold;
			self.hold = self.tmp_block.code;
			self.tmp_block = Block::new(tmp);
		}
	}

	fn soft_drop(&mut self) -> bool {
		if self.shadow_block.pos.1 == self.tmp_block.pos.1 {
			return false;
		}
		self.tmp_block.pos.1 = self.shadow_block.pos.1;
		true
	}

	// return count of lines eliminated
	fn checkline(&self, ln: HashSet<usize>) -> Vec<usize> {
		let mut elims = Vec::new();
		for each_ln in ln.iter() {
			let mut flag = true;
			for x in 0..10 {
				if self.color[*each_ln][x] == 7 {
					flag = false;
				}
			}
			if flag {
				elims.push(*each_ln);
			}
		}
		elims
	}

	fn proc_elim(&mut self, elims: Vec<usize>) {
		if elims.is_empty() {
			self.cm = 0;
			if self.tcm > 0 {
				self.tcm = ATTACK_B2B_INC;
			}
			return;
		}
		let mut movedown = 0;
		for i in 0..40 {
			let mut flag = false;
			for elim in elims.iter() {
				if i == *elim {
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
			self.color[i - movedown] = self.color[i];
		}
	}

	// moving test
	fn test_twist2(&mut self) -> bool {
		self.tmp_block.pos.0 -= 1;
		if self.tmp_block.test(self) {
			self.tmp_block.pos.0 += 1;
			return false;
		}
		self.tmp_block.pos.0 += 2;
		if self.tmp_block.test(self) {
			self.tmp_block.pos.0 -= 1;
			return false;
		}
		self.tmp_block.pos.0 -= 1;
		self.tmp_block.pos.1 += 1;
		if self.tmp_block.test(self) {
			self.tmp_block.pos.1 -= 1;
			return false;
		}
		self.tmp_block.pos.1 -= 1;
		true
	}

	// test all types of twists
	// return 0: none, 1: mini, 2: regular
	fn test_twist(&mut self) -> u32 {
		// No o spin
		if self.tmp_block.code == 3 {
			return 0;
		}
		if !self.test_twist2() {
			return 0;
		}
		// No mini i spin
		if self.tmp_block.code == 0 {
			return 1;
		}
		let tmp = TWIST_MINI_CHECK[self.tmp_block.code as usize]
			[self.tmp_block.rotation as usize];
		for mini_pos in tmp.iter() {
			let check_x = self.tmp_block.pos.0 + mini_pos.0;
			let check_y = self.tmp_block.pos.1 + mini_pos.1;
			if self.color[check_y as usize][check_x as usize] == 7 {
				return 1;
			}
		}
		2
	}

	// true = death
	fn flush_garbage(&mut self, max: usize) -> bool {
		let mut flag = false;
		self.generate_garbage(max);
		if self.calc_shadow() {
			flag = true;
		}
		if self.height == 40 {
			flag = true;
		}
		flag
	}

	// push a new attack into pending garbage queue
	pub fn push_garbage(&mut self, atk: u32) {
		if atk == 0 {
			return;
		}
		self.garbages.push_back(atk);
	}

	// pull pending garbages and write to board color
	fn generate_garbage(&mut self, keep: usize) -> u32 {
		const SAME_LINE: f32 = 0.6;
		let mut ret = 0;
		loop {
			if self.garbages.len() <= keep {
				break;
			}
			let mut count = match self.garbages.pop_front() {
				Some(x) => x,
				None => break,
			} as usize;
			self.height += count as i32;
			let mut slot = self.rg.rng.gen_range(0..10);
			// assert!(count != 0);
			if count > 40 {
				count = 40;
			}
			ret += count;
			for y in (count..40).rev() {
				for x in 0..10 {
					self.color[y][x] = self.color[y - count][x];
				}
			}
			for y in 0..count {
				let same = self.rg.rng.gen::<f32>();
				if same >= SAME_LINE {
					slot = self.rg.rng.gen_range(0..10);
				}
				for x in 0..10 {
					self.color[y][x] = 2; // L = white
				}
				self.color[y][slot] = 7;
				if !self.tmp_block.test(self) {
					self.tmp_block.pos.1 -= 1;
				}
			}
		}
		ret as u32
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
				self.attack_pool = 0;
				break false;
			}
			let popped_lines = self.garbages.pop_front().unwrap();
			self.attack_pool -= popped_lines;
		}
	}

	// providing whether tspin, shape offset and cleared lines
	// change self b2b and attack_pool
	// return atk, cm, tcm
	fn calc_attack(&self, tspin: u32, line_count: u32) -> (u32, u32, u32) {
		let base_atk = ATTACK_BASE[(line_count - 1) as usize];
		let twist_mult = if tspin > 0 {
			ATTACK_BASE_TWIST_MULTIPLIER
				[((tspin - 1) * 7 + self.tmp_block.code as u32) as usize]
		} else {
			10
		};
		let mut total_mult = 10;
		total_mult += self.cm;
		let cm = self.cm + ATTACK_COMBO_INC;
		let tcm = if tspin > 0 || line_count == 4 {
			total_mult += self.tcm;
			self.tcm + ATTACK_B2B_INC
		} else {
			0
		};
		let mut atk = base_atk * twist_mult * total_mult / 1000;
		if self.height == 0 {
			atk += 10;
		}
		(atk, cm, tcm)
	}

	fn set_attack_se(&mut self) {
		if self.attack_pool > 0 {
			if self.tcm == 0 {
				self.last_se = SoundEffect::AttackDrop;
			} else {
				self.last_se = SoundEffect::AttackDrop2;
			}
		} else {
			self.last_se = SoundEffect::ClearDrop;
		} // pc will overwrite this
		if self.height == 0 {
			self.last_se = SoundEffect::PerfectClear;
		}
	}

	// set color, update height
	// return lines to check
	fn hard_drop_set_color(&mut self) -> HashSet<usize> {
		let tmppos = self.tmp_block.getpos();
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
			self.color[py][px] = self.tmp_block.code;
		}
		lines_tocheck
	}

	// true: die
	pub fn hard_drop(&mut self) -> bool {
		// check twist before setting color
		let twist = self.test_twist();
		let lines_tocheck = self.hard_drop_set_color();

		let elim = self.checkline(lines_tocheck);
		let line_count = elim.len() as u32;
		self.proc_elim(elim);
		// put attack amount into pool
		if line_count > 0 {
			self.height -= line_count as i32;
			// assert!(self.attack_pool != 0)
			let ret = self.calc_attack(twist, line_count);
			self.attack_pool = ret.0;
			self.cm = ret.1;
			self.tcm = ret.2;
			self.set_attack_se();
		} else {
			// plain drop: attack execution
			self.last_se = SoundEffect::PlainDrop;
			self.generate_garbage(0); // drain garbage
			if self.height == 40 {
				return true;
			}
		}

		// new block
		self.spawn_block();
		if self.calc_shadow() {
			return true;
		}
		false
	}

	// true = death
	pub fn press_down(&mut self) -> bool {
		if !self.soft_drop() {
			return self.hard_drop();
		} else {
			self.last_se = SoundEffect::SoftDrop;
		}
		false
	}

	// true = death
	pub fn press_up(&mut self) -> bool {
		self.soft_drop();
		self.hard_drop()
	}

	// true: die
	pub fn calc_shadow(&mut self) -> bool {
		self.shadow_block = self.tmp_block.clone();
		loop {
			self.shadow_block.pos.1 -= 1;
			if !self.shadow_block.test(self) {
				self.shadow_block.pos.1 += 1;
				break self.shadow_block.bottom_pos() >= 20;
			}
		}
	}

	pub fn generate_display(&self) -> Display {
		let mut display = Display::new(self.id);
		for i in 0..20 {
			display.color[i] = self.color[i];
		}
		display.shadow_block = self.shadow_block.compress();
		display.tmp_block = self.tmp_block.compress();
		display.garbages = self.garbages.clone();
		display.hold = self.hold;
		display.tmp_block = self.tmp_block.compress();
		display.garbages = self.garbages.clone();
		display.cm = self.cm;
		display.tcm = self.tcm;
		for i in 0..6 {
			display.bag_preview[i] = self.rg.bag[i];
		}
		display
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::test::*;

	#[test]
	fn test_is_pos_inside() {
		let board = Board::new(1);
		assert_eq!(board.is_pos_inside((10, 40)), false);
		assert_eq!(board.is_pos_inside((10, 5)), false);
		assert_eq!(board.is_pos_inside((0, 0)), true);
		assert_eq!(board.is_pos_inside((4, 20)), true);
	}

	#[test]
	fn test_test_twist() {
		let mut board =
			test::generate_solidlines([2, 3, 0, 2, 0, 0, 0, 0, 0, 0]);
		board.color[39][1] = 7; // sdp: (1, 0)
		board.tmp_block = Block::new(1); // █▄▄
		board.tmp_block.pos.0 = 1;
		board.tmp_block.pos.1 = 0;
		board.tmp_block.rotation = 3;
		assert_eq!(board.test_twist(), 2);
	}

	#[test]
	fn test_calc_shadow() {
		let mut board =
			test::generate_solidlines([1, 3, 2, 5, 4, 1, 2, 5, 2, 0]);
		board.tmp_block = Block::new(1); // █▄▄
		assert!(!board.calc_shadow());
		use std::collections::HashSet;
		let mut blocks: HashSet<(i32, i32)> = HashSet::new();
		blocks.insert((3, 6));
		blocks.insert((3, 5));
		blocks.insert((4, 5));
		blocks.insert((5, 5));
		let shadow_pos = board.shadow_block.getpos();
		println!("{:?} {:?}", blocks, shadow_pos);
		for i in 0..4 {
			blocks.remove(&(shadow_pos[i].0 as i32, shadow_pos[i].1 as i32));
		}
		assert!(blocks.is_empty());
		board.move2(-1); // move to very left
		assert!(!board.calc_shadow());

		blocks.insert((0, 4));
		blocks.insert((0, 3));
		blocks.insert((1, 3));
		blocks.insert((2, 3));
		let shadow_pos = board.shadow_block.getpos();
		println!("{:?} {:?}", blocks, shadow_pos);
		for i in 0..4 {
			blocks.remove(&(shadow_pos[i].0 as i32, shadow_pos[i].1 as i32));
		}
	}
}
