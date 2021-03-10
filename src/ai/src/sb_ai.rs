// slightly better AI
use tttz_protocol::Display;
use tttz_protocol::KeyType;
use tttz_ruleset::*;

use crate::ai_utils::*;
use crate::ai::Thinker;

use std::collections::VecDeque;

pub struct SbAi {}

impl SbAi {
	pub fn new() -> Self {
		SbAi {}
	}
}

impl Thinker for SbAi {
	fn main_think(&mut self, display: &Display) -> VecDeque<KeyType> {
		let mut ret = VecDeque::new();
		if display.hold == 7 {
			ret.push_back(KeyType::Hold);
			return ret
		}

		let (heights, highest_hole_x, _highest_hole) = get_height_and_hole(&display);

		ret
	}
}
