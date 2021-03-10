// slightly better AI
// observe one block forward
use tttz_protocol::{Display, KeyType, BoardMsg};
use tttz_ruleset::*;
use tttz_mpboard::{Block, Board};

use crate::ai_utils::*;
use crate::ai::Thinker;

use std::collections::VecDeque;

pub struct SbAi {
	test_board: Board,
}

impl SbAi {
	pub fn new() -> Self {
		SbAi {
			test_board: Board::new(0),
		}
	}

	// think2 is called for the given first block
	// return value
	fn think2(&mut self, twist: bool, block: Block) -> f32 {
		0.0
	}
}

impl Thinker for SbAi {
	fn main_think(&mut self, display: Display) -> VecDeque<KeyType> {
		let mut ret = VecDeque::new();
		if display.hold == 7 {
			ret.push_back(KeyType::Hold);
			return ret
		}

		let (heights, highest_hole_x, _highest_hole) = get_height_and_hole(&display);
		self.test_board.rg.bag = display.bag_preview.iter().map(|x| *x).collect();
		self.test_board.display = display;

		let max_value: f32 = -f32::NEG_INFINITY;
		let mut best_id: u8 = 0;
		let mut best_rotation_before_drop: u8 = 0;
		let mut best_dx: i32 = -1;
		let mut best_key_after_drop: KeyType = KeyType::Nothing;
		// we should not really optimize first block since
		// possible twists can be filtered out easily
		for (id, option_code) in [
			self.test_board.display.tmp_block[2],
			self.test_board.display.hold,
		].iter()
		.map(|x| *x)
		.enumerate() {
			self.test_board.tmp_block.code = option_code;
			for rot in 0..4 {
				let (possible_pos, posx, posy) = convolve_height(&heights, option_code, rot);
				for (dx, height) in possible_pos
					.iter()
					.map(|(x, y)| (*x as i32, *y as i32)) {
					// we further try rotate and move
					self.test_board.tmp_block.pos.0 = dx;
					self.test_board.tmp_block.pos.1 = height;
					// too high, just skip
					if height <= 1 { continue }
					assert!(self.test_board.tmp_block.test(&self.test_board));

					// generate possible blocks
					let revert_block = self.test_board.tmp_block.clone();
					// now we are iterating the operations after soft drop
					for key_type in [
						KeyType::Left,
						KeyType::Right,
						KeyType::Rotate,
						KeyType::RotateReverse,
						KeyType::RotateFlip,
					].iter() {
						// TODO: handle_msg is an interface for interactive gaming thus inefficient
						self.test_board.handle_msg(BoardMsg::KeyEvent(*key_type));
						let mut flag = true;
						// completely failed operation
						if self.test_board.tmp_block.pos.0 == revert_block.pos.0 &&
							self.test_board.tmp_block.rotation == revert_block.rotation {
							flag = false;
						}

						// can move up/down
						self.test_board.tmp_block.pos.1 += 1;
						if self.test_board.tmp_block.test(&self.test_board) {
							flag = false;
						}
						self.test_board.tmp_block.pos.1 -= 2;
						if self.test_board.tmp_block.test(&self.test_board) {
							flag = false;
						}

						if flag {
							self.test_board.tmp_block.pos.1 += 1;
							// left/right move for twist test
							// we won't make a mini-twist test for efficiency
							self.test_board.tmp_block.pos.0 -= 1;
							let twist = if self.test_board.tmp_block.test(&self.test_board) {
								self.test_board.tmp_block.pos.0 += 1;
								false
							} else {
								self.test_board.tmp_block.pos.0 += 2;
								if self.test_board.tmp_block.test(&self.test_board) {
									self.test_board.tmp_block.pos.0 -= 1;
									false
								} else {
									self.test_board.tmp_block.pos.0 -= 1;
									true
								}
							};
							let block = std::mem::replace(
								&mut self.test_board.tmp_block,
								revert_block.clone()
							);
							let value = self.think2(twist, block);
							if value > max_value {
								best_id = id as u8;
								best_rotation_before_drop = rot;
								best_dx = dx;
								best_key_after_drop = *key_type;
							}
						} else {
							self.test_board.tmp_block = revert_block.clone();
						}
					}
				}
			}
		}
		ret
	}
}
