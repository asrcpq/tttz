// slightly better AI
// observe one block forward
use tttz_protocol::Display;
use tttz_protocol::KeyType;
use tttz_ruleset::*;
use tttz_mpboard::Board;

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

	// the second block needs heavy optimization
	// return value
	fn think2(&mut self) -> f32 {
		let (heights, highest_hole_x, _highest_hole) =
			get_height_and_hole(&self.test_board.display);
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

		// we should not really optimize first block since
		// possible twists can be filtered out easily
		for (id, option_code) in [display.tmp_block[2], display.hold]
			.iter()
			.map(|x| *x)
			.enumerate() {
			for rot in 0..4 {
				let (possible_pos, posx, posy) = convolve_height(&heights, option_code, rot);
				for (dx, height) in possible_pos
					.iter()
					.map(|(x, y)| (*x as i32, *y as i32)) {
					// we further try rotate and move
				}
			}
		}
		self.think2();
		ret
	}
}
