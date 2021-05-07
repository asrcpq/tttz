use tttz_protocol::{BoardMsg, BoardReply, IdType, KeyType};
use tttz_protocol::{Display, Piece};
use tttz_ruleset::*;

use crate::garbage_attack_manager::GarbageAttackManager;
use crate::random_generator::RandomGenerator;
use crate::replay::Replay;
use crate::Field;

pub struct Board {
	pub(in crate) floating_block: Piece,
	shadow_block: Piece,
	pub(in crate) rg: RandomGenerator,
	// pub(in crate) field: Vec<[u8; 10]>,
	pub(in crate) field: Field,
	hold: CodeType,
	gaman: GarbageAttackManager,
	pub replay: Replay,
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
			BoardMsg::Attacked(width, amount) => {
				self.gaman.push_garbage(width, amount);
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
		let code = self.rg.get_code();
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

	// -1 = death
	fn flush_garbage(&mut self, max: usize) -> i32 {
		let garbage_line = self.generate_garbage(max);
		if self.calc_shadow() {
			return -1;
		}
		if self.field.height == 40 {
			return -1;
		}
		garbage_line as i32
	}

	// pull pending garbages and write to board field
	pub fn generate_garbage(&mut self, keep: usize) -> i32 {
		const SAME_LINE: f32 = 0.6;
		let mut ret = 0i32;
		loop {
			if self.gaman.garbages.len() <= keep {
				break;
			}
			let (w, mut count) = match self.gaman.garbages.pop_front() {
				Some((x, y)) => (x, y as usize),
				None => break,
			};
			// assert!(count != 0);
			if count > 40 {
				count = 40;
			}
			for y in (count..40).rev() {
				for x in 0..10 {
					self.field[y][x] = self.field[y - count][x];
				}
			}
			let mut slot = self.rg.get_slot(w); // initial pos
			for y in 0..count {
				let same = self.rg.get_shift();
				if same >= SAME_LINE {
					slot = self.rg.get_slot(w);
				}
				for x in 0..10 {
					self.field[y][x] = b'g';
				}
				for i in 0..w {
					self.field[y][slot as usize + i as usize] = b' ';
				}
				if !self.field.test(&self.floating_block) {
					self.floating_block.pos.1 += 1;
				}
			}
			self.field.height += count as i32;
			ret += count as i32;
			if self.field.height >= 40 {
				ret = -1;
			}
		}
		ret
	}

	fn hard_drop(&mut self) -> BoardReply {
		// check twist before setting field
		let twist = self.field.test_twist(&mut self.floating_block);
		let line_count = self.field.settle_block(&self.floating_block);
		// gaman will safely ignore pc when lc = 0
		let (raw_atk, atk) = self.gaman.calc_attack(
			twist,
			line_count,
			self.floating_block.code,
			self.field.height == 0,
		);
		if line_count > 0 {
			self.spawn_block();
			if self.calc_shadow() {
				return BoardReply::Die;
			}
			BoardReply::ClearDrop(line_count, atk, raw_atk)
		} else {
			// plain drop: attack execution
			let ret = self.generate_garbage(0);
			// ret=-1 <=> height=40
			if ret == -1 {
				return BoardReply::Die;
			}
			self.spawn_block();
			if self.calc_shadow() {
				return BoardReply::Die;
			}
			BoardReply::PlainDrop(ret as u32)
		}
	}

	// true = death
	fn press_down(&mut self) -> BoardReply {
		if self.soft_drop() {
			BoardReply::Ok
		} else {
			BoardReply::BadMove
		}
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

	// when no garbage flush everything is simple
	pub fn update_from_display_minor(&mut self, display: &Display) {
		self.rg.bag = display.bag_preview.clone().to_vec();
		self.rg.bag_id = 0;
		self.rg.bag.push(7); // prevent get method generating new bag
	}

	// used for client rendering
	pub fn update_from_display(&mut self, display: &Display) {
		self.hold = display.hold;
		self.shadow_block = display.shadow_block.clone();
		self.floating_block = display.floating_block.clone();
		self.gaman.read_display(display);
		self.rg.bag = display.bag_preview.clone().to_vec();
		self.rg.bag_id = 0;
		self.rg.bag.push(7); // prevent get method generating new bag
		for i in 0..20 {
			self.field[i] = display.color[i];
		}
	}

	pub fn generate_display(
		&self,
		id: IdType,
		seq: usize,
		board_reply: BoardReply,
	) -> Display {
		let mut display = Display {
			seq,
			id,
			color: self.field.iter().take(20).cloned().collect(),
			shadow_block: self.shadow_block.clone(),
			floating_block: self.floating_block.clone(),
			hold: self.hold,
			bag_preview: self.rg.preview_code(),
			cm: 0,
			tcm: 0,
			garbages: Default::default(),
			board_reply,
		};
		self.gaman.write_display(&mut display);
		display
	}

	pub fn save_replay(
		&mut self,
		filename: &str,
	) -> Result<bool, Box<dyn std::error::Error>> {
		self.replay.save(filename, &mut self.rg)
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
		board.floating_block = Piece::new(0);
		board.floating_block.pos.0 = 0;
		board.floating_block.pos.1 = 0;
		board.floating_block.rotation = 1;
		board.rotate(1);
		assert_eq!(board.floating_block.rotation, 2);
		assert_eq!(board.floating_block.pos, (0, 0));
		assert_eq!(board.field.test_twist(&mut board.floating_block), 1);
		board.floating_block.rotation = 3;
		board.rotate(1);
		assert_eq!(board.floating_block.rotation, 0);
		assert_eq!(board.floating_block.pos, (0, 0));
		assert_eq!(board.field.test_twist(&mut board.floating_block), 1);
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
			eprintln!("height: {}", board.field.height);
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
				eprintln!("height: {}", board.field.height);
			}
		}
		assert_eq!(board.field.height, 0);
	}
}
