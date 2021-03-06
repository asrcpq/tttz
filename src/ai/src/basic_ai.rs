// stupid ai, put block to make least holes and lowest height
use tttz_protocol::Display;
use tttz_protocol::KeyType;
use tttz_ruleset::*;

use crate::ai::Thinker;
use crate::ai_utils::*;

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

impl Default for BasicAi {
	fn default() -> Self {
		BasicAi {
			cover_weight: 0.5,
			hole_weight: 1.0,
			height_weight: 1.0,
		}
	}
}

impl Thinker for BasicAi {
	fn reset(&mut self) {}

	fn main_think(&mut self, display: Display) -> VecDeque<KeyType> {
		if display.hold == 7 {
			let mut ret = VecDeque::new();
			ret.push_back(KeyType::Hold);
			return ret;
		}

		let (heights, highest_hole_x, _highest_hole) =
			get_height_and_hole(&display);

		let mut best_score = f32::INFINITY;
		let mut best_rotation = 0;
		let mut best_posx = 0;
		let mut best_id = 0;
		for (id, &option_code) in
			[display.floating_block.code, display.hold].iter().enumerate()
		{
			for rot in 0..4 {
				let (possible_pos, posx, posy) =
					convolve_height(&heights, option_code, rot);
				for (dx, height) in
					possible_pos.iter().map(|&(x, y)| (x as PosType, y as PosType))
				{
					let mut delta_heights = [0; 4];
					let mut block_count = [0; 4];
					for block in 0..4 {
						let dh = posy[block] + height as PosType
							- heights[dx as usize + posx[block] as usize] as PosType;
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
						&& dx + BLOCK_WIDTH[option_code as usize][rot as usize]
							> highest_hole_x) as PosType;
					let score = (height as f32
						+ BLOCK_MCH[option_code as usize][rot as usize])
						* self.height_weight + hole as f32
						* self.hole_weight + cover as f32
						* self.cover_weight;
					if score < best_score {
						// eprintln!(
						// 	"{} {} {} = {} overtake {} at dx: {}, rot: {}",
						// 	height, hole, cover, score, best_score, dx, rot,
						// );
						best_score = score;
						best_rotation = rot;
						best_posx = dx;
						best_id = id;
					}
				}
			}
		}
		let best_code = if best_id == 0 {
			display.floating_block.code
		} else {
			// best solution is from the hold block
			display.hold
		};
		// perform action
		generate_keys(GenerateKeyParam {
			hold_swap: best_id != 0,
			code: best_code,
			rotation: best_rotation,
			post_key: KeyType::Nothing,
			dx: best_posx,
		})
	}
}
