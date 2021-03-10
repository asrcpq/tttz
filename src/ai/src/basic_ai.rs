// stupid ai, put block to make least holes and lowest height
use tttz_ruleset::*;
use tttz_protocol::Display;
use tttz_protocol::KeyType;

use crate::ai_utils::*;
use crate::ai::Thinker;

use std::collections::VecDeque;

// A hole is a group of vertical continuous blank blocks
pub struct BasicAi {
	// how bad it is to put a block on the highest hole
	pub cover_weight: f32,
	// how bad it is to create a new hole
	pub hole_weight: f32,
	// how bad it is to increase height
	pub height_weight: f32,
}

impl BasicAi {
	pub fn new() -> Self {
		BasicAi {
			cover_weight: 0.,
			hole_weight: 1.0,
			height_weight: 1.0,
		}
	}
}

impl Thinker for BasicAi {
	fn main_think(&mut self, display: &Display) -> VecDeque<KeyType> {
		let mut ret = VecDeque::new();

		if display.hold == 7 {
			ret.push_back(KeyType::Hold);
			return ret;
		}

		let (heights, highest_hole_x, _highest_hole) = get_height_and_hole(&display);

		let mut best_score: f32 = 0.0;
		let mut best_rotation = 0;
		let mut best_posx = 0;
		let mut best_id = 0;
		for (id, option_code) in [display.tmp_block[2], display.hold].iter().enumerate()
		{
			for rot in 0..4 {
				let mut dx = 0;
				loop {
					if dx + BLOCK_WIDTH[*option_code as usize][rot as usize]
						> 10
					{
						break;
					}

					let mut posx = [0; 4];
					let mut posy = [0; 4];
					for block in 0..4 {
						let tmp = BPT[*option_code as usize][rot as usize][block as usize];
						posx[block as usize] = tmp.0;
						posy[block as usize] = tmp.1;
					}
					let mut posy_sum = 0;
					for each_posy in posy.iter() {
						posy_sum += each_posy;
					}
					let mut height = 0;
					'movedown_check: loop {
						for block in 0..4 {
							if posy[block] + height
								== (heights[dx as usize + posx[block] as usize])
									as i32
							{
								height -= 1;
								break 'movedown_check;
							}
						}
						height += 1;
					}

					let mut delta_heights = [0; 4];
					let mut block_count = [0; 4];
					for block in 0..4 {
						let dh = heights[dx as usize + posx[block] as usize] as i32
							- posy[block] - height;
						block_count[posx[block] as usize] += 1;
						if dh > delta_heights[posx[block] as usize] {
							delta_heights[posx[block] as usize] = dh;
						}
					}
					let mut hole: i32 = 0;
					for block in 0..4 {
						if delta_heights[block] > block_count[block] {
							hole += 1;
						}
					}
					let cover = (dx <= highest_hole_x
						&& dx
							+ BLOCK_WIDTH[*option_code as usize][rot as usize]
							> highest_hole_x) as i32;
					let score = (
							height as f32 +
							MCH[*option_code as usize][rot as usize]
						) * self.height_weight -
						hole as f32 * self.hole_weight -
						cover as f32 * self.cover_weight;
					if score > best_score {
						eprintln!(
							"{} {} {} = {} overtake {} at dx: {}, rot: {}",
							height, hole, cover, score, best_score, dx, rot,
						);
						best_score = score;
						best_rotation = rot;
						best_posx = dx;
						best_id = id;
					} 
					dx += 1;
				}
			}
		}

		let best_code = if best_id == 0 {
			display.tmp_block[2]
		} else {
			// best solution is from the hold block
			ret.push_back(KeyType::Hold);
			display.hold
		};
		// perform action
		let current_posx = INITIAL_POS[best_code as usize];
		let rotated_pos0 =
			current_posx + SRP[best_code as usize][best_rotation as usize].0;
		let (keycode, times) = if best_posx == 0 {
			(KeyType::LLeft, 1)
		} else if best_posx
			== 10 - BLOCK_WIDTH[best_code as usize][best_rotation as usize]
		{
			(KeyType::RRight, 1)
		} else if rotated_pos0 > best_posx {
			(KeyType::Left, rotated_pos0 - best_posx)
		} else {
			(KeyType::Right, best_posx - rotated_pos0)
		};
		if best_rotation == 1 {
			ret.push_back(KeyType::Rotate);
		} else if best_rotation == 3 {
			ret.push_back(KeyType::RotateReverse);
		} else if best_rotation == 2 {
			ret.push_back(KeyType::RotateFlip);
		}
		for _ in 0..times {
			ret.push_back(keycode.clone());
		}
		ret.push_back(KeyType::HardDrop);
		ret
	}
}
