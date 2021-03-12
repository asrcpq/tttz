use tttz_ruleset::*;
use tttz_protocol::Display;

use std::collections::VecDeque;

pub struct GarbageAttackManager {
	cm: u32,
	tcm: u32,
	pub garbages: VecDeque<u32>,
}

impl Default for GarbageAttackManager {
	fn default() -> Self {
		GarbageAttackManager {
			cm: 0,
			tcm: 0,
			garbages: VecDeque::new(),
		}
	}
}

impl GarbageAttackManager {
	// push a new attack into pending garbage queue
	pub fn push_garbage(&mut self, atk: u32) {
		if atk == 0 {
			return;
		}
		self.garbages.push_back(atk);
	}

	// return atk
	fn counter_attack(&mut self, mut atk: u32) -> u32 {
		loop {
			// return if attack remains
			if self.garbages.is_empty() {
				break atk;
			}
			if self.garbages[0] >= atk {
				self.garbages[0] -= atk;
				if self.garbages[0] == 0 {
					self.garbages.pop_front();
				}
				break 0;
			}
			let popped_lines = self.garbages.pop_front().unwrap();
			atk -= popped_lines;
		}
	}

	pub fn calc_attack(
		&mut self,
		tspin: u32,
		line_count: u32,
		code: u8,
		pc: bool,
	) -> u32 {
		if line_count == 0 {
			self.cm = 0;
			if self.tcm > 0 {
				self.tcm = ATTACK_B2B_INC;
			}
			return 0;
		}
		let base_atk = ATTACK_BASE[(line_count - 1) as usize];
		let twist_mult = if tspin > 0 {
			ATTACK_BASE_TWIST_MULTIPLIER
				[(tspin as usize - 1) * 7 + code as usize]
		} else {
			10
		};
		let mut total_mult = 10;
		total_mult += self.cm;
		self.cm += ATTACK_COMBO_INC;
		self.tcm = if tspin > 0 || line_count == 4 {
			total_mult += self.tcm;
			self.tcm + ATTACK_B2B_INC
		} else {
			0
		};
		let mut atk = base_atk * twist_mult * total_mult / 1000;
		eprintln!("Attack parts: {} {} {}", base_atk, twist_mult, total_mult);
		if pc {
			atk += 10;
		}
		let atk = self.counter_attack(atk);
		atk
	}

	pub fn set_display(&self, mut display: &mut Display) {
		display.garbages = self.garbages.clone();
		display.cm = self.cm;
		display.tcm = self.tcm;
	}
}
