use tttz_protocol::{Piece, Display};
use tttz_protocol::{BoardMsg, BoardReply, IdType, KeyType};
use tttz_ruleset::*;

use crate::Field;
use crate::garbage_attack_manager::GarbageAttackManager;
use crate::random_generator::RandomGenerator;
use crate::replay::Replay;
use rand::Rng;

use std::collections::HashSet;

pub struct Board {
	pub(in crate) floating_block: Piece,
	shadow_block: Piece,
	pub(in crate) rg: RandomGenerator,
	// pub(in crate) field: Vec<[u8; 10]>,
	pub(in crate) field: Field,
	hold: CodeType,
	gaman: GarbageAttackManager,
	height: i32,
	pub replay: Replay,
}

use std::fmt;
impl fmt::Debug for Board {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		for row in self.field.iter().rev() {
			for &ch in row.iter() {
				write!(f, "{} ", if ch == b' ' { '0' } else { ch as char })?;
			}
			writeln!(f)?;
		}
		Ok(())
	}
}

impl Default for Board {
	fn default() -> Board {
		let replay = Default::default();
		let mut board = Board {
			floating_block: Piece::new(0), // immediately overwritten
			shadow_block: Piece::new(0),   // immediately overwritten
			rg: Default::default(),
			field: Default::default(),
			hold: 7,
			gaman: Default::default(),
			height: 0,
			replay,
		};
		board.spawn_block();
		board.calc_shadow();
		board
	}
}

impl Board {
	fn move1(&mut self, dx: i8) -> bool {
		self.floating_block.pos.0 += dx;
		if !self.field.test(&self.floating_block) {
			self.floating_block.pos.0 -= dx;
			return false;
		}
		true
	}

	fn move2(&mut self, dx: i8) -> bool {
		if self.move1(dx) {
			while self.move1(dx) {}
			true
		} else {
			false
		}
	}

	fn rotate(&mut self, dr: i8) -> BoardReply {
		let revert_block = self.floating_block.clone();
		let ret = self.field.rotate(&mut self.floating_block, dr);
		if ret == 0 {
			self.floating_block = revert_block;
		}
		self.calc_shadow();
		match ret {
			1 => BoardReply::Ok,
			2 => BoardReply::RotateTwist,
			_ => BoardReply::BadMove,
		}
	}

	pub fn handle_msg(&mut self, board_msg: BoardMsg) -> BoardReply {
		self.replay.push_operation(board_msg.clone());
		let mut okflag = true;
		match board_msg {
			BoardMsg::KeyEvent(key_type) => match key_type {
				KeyType::HardDrop => return self.press_up(),
				KeyType::SonicDrop => return self.press_down(),
				KeyType::RotateReverse => return self.rotate(-1),
				KeyType::Rotate => return self.rotate(1),
				KeyType::RotateFlip => return self.rotate(2),
				KeyType::Nothing => {}
				KeyType::Hold => {
					self.hold();
				}
				KeyType::Left => {
					okflag = self.move1(-1);
				}
				KeyType::LLeft => {
					okflag = self.move2(-1);
				}
				KeyType::Right => {
					okflag = self.move1(1);
				}
				KeyType::RRight => {
					okflag = self.move2(1);
				}
			},
			BoardMsg::Attacked(amount) => {
				self.gaman.push_garbage(amount);
				const MAX_GARBAGE_LEN: usize = 5;
				if self.gaman.garbages.len() > MAX_GARBAGE_LEN {
					let ret = self.flush_garbage(MAX_GARBAGE_LEN);
					if ret == -1 {
						return BoardReply::Die;
					} else {
						return BoardReply::GarbageOverflow(ret as u32);
					}
				}
			}
		}
		self.calc_shadow();
		if okflag {
			BoardReply::Ok
		} else {
			BoardReply::BadMove
		}
	}

	pub fn spawn_block(&mut self) {
		let code = self.rg.get();
		self.replay.push_block(code);
		self.floating_block = Piece::new(code);
	}

	fn hold(&mut self) {
		if self.hold == 7 {
			self.hold = self.floating_block.code;
			self.spawn_block();
		} else {
			let tmp = self.hold;
			self.hold = self.floating_block.code;
			self.floating_block = Piece::new(tmp);
		}
	}

	fn soft_drop(&mut self) -> bool {
		if self.shadow_block.pos.1 == self.floating_block.pos.1 {
			return false;
		}
		self.floating_block.pos.1 = self.shadow_block.pos.1;
		true
	}

	// return count of lines eliminated
	fn checkline(&self, ln: HashSet<usize>) -> Vec<usize> {
		let mut elims = Vec::new();
		for &each_ln in ln.iter() {
			let mut flag = true;
			for x in 0..10 {
				if self.field[each_ln][x] == b' ' {
					flag = false;
				}
			}
			if flag {
				elims.push(each_ln);
			}
		}
		elims
	}

	fn proc_elim(&mut self, elims: Vec<usize>) {
		let mut movedown = 0;
		for i in 0..40 {
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
			self.field[i - movedown] = self.field[i];
		}
	}

	// -1 = death
	fn flush_garbage(&mut self, max: usize) -> i32 {
		let garbage_line = self.generate_garbage(max);
		self.height += garbage_line;
		if self.calc_shadow() {
			return -1;
		}
		if self.height == 40 {
			return -1;
		}
		garbage_line as i32
	}

	// set field, update height
	// return lines to check
	fn hard_drop_set_field(&mut self) -> HashSet<usize> {
		let tmppos = self.floating_block.getpos();
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
			self.field[py][px] = ID_TO_CHAR[self.floating_block.code as usize];
		}
		lines_tocheck
	}

	// pull pending garbages and write to board field
	pub fn generate_garbage(&mut self, keep: usize) -> i32 {
		const SAME_LINE: f32 = 0.6;
		let mut ret = 0;
		loop {
			if self.gaman.garbages.len() <= keep {
				break;
			}
			let mut count = match self.gaman.garbages.pop_front() {
				Some(x) => x,
				None => break,
			} as usize;
			let mut slot = self.rg.rng.gen_range(0..10);
			// assert!(count != 0);
			if count > 40 {
				count = 40;
			}
			ret += count;
			for y in (count..40).rev() {
				for x in 0..10 {
					self.field[y][x] = self.field[y - count][x];
				}
			}
			for y in 0..count {
				let same = self.rg.rng.gen::<f32>();
				if same >= SAME_LINE {
					slot = self.rg.rng.gen_range(0..10);
				}
				for x in 0..10 {
					self.field[y][x] = b'g';
				}
				self.field[y][slot] = b' ';
				if !self.field.test(&self.floating_block) {
					self.floating_block.pos.1 -= 1;
				}
			}
		}
		ret as i32
	}

	fn hard_drop(&mut self) -> BoardReply {
		// check twist before setting field
		let twist = self.field.test_twist(&mut self.floating_block);
		let lines_tocheck = self.hard_drop_set_field();

		let elim = self.checkline(lines_tocheck);
		let line_count = elim.len() as u32;
		self.proc_elim(elim);
		// put attack amount into pool
		let atk = self.gaman.calc_attack(
			twist,
			line_count,
			self.floating_block.code,
			self.height == line_count as i32,
		);
		if line_count > 0 {
			self.height -= line_count as i32;
			self.spawn_block();
			self.calc_shadow(); // cannot die from a clear drop!
			BoardReply::ClearDrop(line_count, atk)
		} else {
			// plain drop: attack execution
			let ret = self.generate_garbage(0);
			// ret=-1 <=> height=40
			if ret == -1 {
				return BoardReply::Die;
			}
			self.height += ret;
			self.spawn_block();
			if self.calc_shadow() {
				return BoardReply::Die;
			}
			BoardReply::PlainDrop(ret as u32)
		}
	}

	// true = death
	fn press_down(&mut self) -> BoardReply {
		if !self.soft_drop() {
			return self.hard_drop();
		}
		BoardReply::Ok
	}

	// true = death
	fn press_up(&mut self) -> BoardReply {
		self.soft_drop();
		self.hard_drop()
	}

	// true: die
	pub(in crate) fn calc_shadow(&mut self) -> bool {
		self.shadow_block = self.floating_block.clone();
		loop {
			self.shadow_block.pos.1 -= 1;
			if !self.field.test(&self.shadow_block) {
				self.shadow_block.pos.1 += 1;
				break self.shadow_block.pos.1 >= 20;
			}
		}
	}

	pub fn generate_display(&self, id: IdType, board_reply: BoardReply) -> Display {
		let mut display = Display {
			id,
			color: self.field.iter().take(20).cloned().collect(),
			shadow_block: self.shadow_block.clone(),
			floating_block: self.floating_block.clone(),
			hold: self.hold,
			bag_preview: self.rg.bag.iter().take(6).cloned().collect(),
			cm: 0,
			tcm: 0,
			garbages: Default::default(),
			board_reply,
		};
		self.gaman.set_display(&mut display);
		for i in 0..6 {
			display.bag_preview[i] = self.rg.bag[i];
		}
		display
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::utils::*;

	#[test]
	fn test_test_tspin() {
		let mut board =
			test::generate_solidlines([1, 0, 3, 0, 0, 0, 0, 0, 0, 0]);
		board.field[38][2] = b' ';
		board.floating_block = Piece::new(5);
		board.floating_block.pos.0 = 0;
		board.floating_block.pos.1 = 0;
		board.floating_block.rotation = 2;
		assert_eq!(board.field.test_twist(&mut board.floating_block), 2);
	}

	#[test]
	fn test_jl_twist() {
		// The famous 180
		let mut board =
			test::generate_solidlines([2, 3, 0, 3, 2, 0, 0, 0, 0, 0]);
		board.field[39][1] = b' ';
		board.field[39][3] = b' ';
		board.floating_block = Piece::new(1);
		board.floating_block.pos.0 = 1;
		board.floating_block.pos.1 = 0;
		board.floating_block.rotation = 3;
		assert_eq!(board.field.test_twist(&mut board.floating_block), 2);
		board.floating_block.code = 2;
		board.floating_block.pos.0 = 2;
		board.floating_block.rotation = 1;
		assert_eq!(board.field.test_twist(&mut board.floating_block), 2);

		// It is a regular twist, as long as its center is blocked
		let mut board =
			test::generate_solidlines([2, 2, 0, 2, 2, 0, 0, 0, 0, 0]);
		board.field[0][1] = b' ';
		board.field[0][3] = b' ';
		board.floating_block = Piece::new(1);
		board.floating_block.pos.0 = 1;
		board.floating_block.pos.1 = 0;
		board.floating_block.rotation = 3;
		assert_eq!(board.field.test_twist(&mut board.floating_block), 2);
		board.floating_block.code = 2;
		board.floating_block.pos.0 = 2;
		board.floating_block.rotation = 1;
		assert_eq!(board.field.test_twist(&mut board.floating_block), 2);

		// mini-twist
		let mut board =
			test::generate_solidlines([2, 3, 0, 0, 3, 2, 0, 0, 0, 0]);
		board.field[0][1] = b' ';
		board.field[0][4] = b' ';
		board.floating_block = Piece::new(1);
		board.floating_block.pos.0 = 2;
		board.floating_block.pos.1 = 0;
		board.floating_block.rotation = 0;
		assert_eq!(board.field.test_twist(&mut board.floating_block), 1);
		board.floating_block.code = 2;
		board.floating_block.pos.0 = 1;
		assert_eq!(board.field.test_twist(&mut board.floating_block), 1);

		// no twist
		let mut board =
			test::generate_solidlines([2, 1, 1, 1, 2, 2, 2, 2, 2, 2]);
		board.floating_block = Piece::new(1);
		board.floating_block.pos.0 = 1;
		board.floating_block.pos.1 = 1;
		board.floating_block.rotation = 0;
		assert_eq!(board.field.test_twist(&mut board.floating_block), 0);
		board.floating_block.code = 2;
		assert_eq!(board.field.test_twist(&mut board.floating_block), 0);

		// in-place 180 kick
		let mut board =
			test::generate_solidlines([4, 0, 0, 4, 2, 2, 2, 2, 2, 2]);
		board.field[3][2] = b'i';
		board.floating_block = Piece::new(1);
		board.floating_block.pos.0 = 1;
		board.floating_block.pos.1 = 0;
		board.floating_block.rotation = 1;
		assert_eq!(board.field.test_twist(&mut board.floating_block), 1);
		board.rotate(2);
		assert_eq!(board.floating_block.pos, (1, 0));
		assert_eq!(board.field.test_twist(&mut board.floating_block), 1);
		board.floating_block.code = 2;
		board.floating_block.rotation = 1;
		board.rotate(2);
		assert_eq!(board.floating_block.pos, (1, 0));
		assert_eq!(board.field.test_twist(&mut board.floating_block), 1);
	}

	#[test]
	fn test_i_kick() {
		let mut board: Board = Default::default();
		board.floating_block = Piece::new(0);
		board.floating_block.pos.0 = 3;
		board.floating_block.pos.1 = 5;
		board.floating_block.rotation = 0;
		assert_eq!(board.field.test_twist(&mut board.floating_block), 0);

		board.floating_block.pos.1 = 0;
		board.rotate(2);
		assert_eq!(board.floating_block.pos.1, 0);
		board.rotate(2);
		assert_eq!(board.floating_block.pos.1, 1);

		let mut board =
			test::generate_solidlines([0, 4, 4, 4, 1, 0, 0, 0, 0, 0]);
		for i in 1..4 {
			board.field[0][i] = b' ';
		}
		eprintln!("{:?}", board);
		board.floating_block = Piece::new(0);
		board.floating_block.pos.0 = 0;
		board.floating_block.pos.1 = 0;
		board.floating_block.rotation = 1;
		board.rotate(1);
		assert_eq!(board.floating_block.rotation, 2);
		assert_eq!(board.floating_block.pos, (0, 0));
		assert_eq!(board.field.test_twist(&mut board.floating_block), 2);
		board.floating_block.rotation = 3;
		board.rotate(1);
		assert_eq!(board.floating_block.rotation, 0);
		assert_eq!(board.floating_block.pos, (0, 0));
		assert_eq!(board.field.test_twist(&mut board.floating_block), 2);
	}

	#[test]
	fn test_calc_shadow() {
		let mut board =
			test::generate_solidlines([1, 3, 2, 5, 4, 1, 2, 5, 2, 0]);
		board.floating_block = Piece::new(1); // █▄▄
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

	#[test]
	fn test_shadow_die() {
		let mut board =
			test::generate_solidlines([1, 20, 20, 19, 0, 0, 0, 0, 0, 0]);
		board.floating_block = Piece::new(1);
		board.floating_block.pos.0 = 1;
		assert!(board.calc_shadow());
		board.floating_block.rotation = 2;
		assert!(!board.calc_shadow());
	}

	#[test]
	fn test_pc() {
		let mut board: Board = Default::default();
		test::oracle(&mut board, 0, &[0; 10]);
		eprintln!("{:?}", board.rg.bag);
		for _ in 0..4 {
			board.press_up();
			eprintln!("height: {}", board.height);
		}
		for t in -1..=1 {
			if t == 0 {
				continue;
			}
			for i in 0..3 {
				// eprintln!("{:?}", board);
				board.rotate(1);
				board.move2(t);
				for _ in 0..i {
					board.move1(-t);
				}
				board.calc_shadow();
				board.press_up();
				eprintln!("height: {}", board.height);
			}
		}
		assert_eq!(board.height, 0);
	}
}
