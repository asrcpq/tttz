use tttz_ruleset::*;

use std::collections::VecDeque;

pub struct GarbageAttackManager {
	pub cm: u32,
	pub tcm: u32,
	pub garbages: VecDeque<u32>,
	pub attack_pool: u32,
}

impl Default for GarbageAttackManager {
	fn default() -> Self {
		GarbageAttackManager {
			cm: 0,
			tcm: 0,
			garbages: VecDeque::new(),
			attack_pool: 0,
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
	pub fn calc_attack(
		&mut self,
		tspin: u32,
		line_count: u32,
		code: u8,
		pc: bool,
	) {
		if line_count == 0 {
			self.cm = 0;
			if self.tcm > 0 {
				self.tcm = ATTACK_B2B_INC;
			}
			self.attack_pool = 0;
			return;
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
		if pc {
			atk += 10;
		}
		self.attack_pool = atk;
	}
}
