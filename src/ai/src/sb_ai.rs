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
		VecDeque::new()
	}
}
