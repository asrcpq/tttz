use tttz_protocol::Display;
use tttz_ruleset::*;

use std::collections::VecDeque;

#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct GarbageAttackManager {
	cm: u32,
	tcm: u32,
	pub garbages: VecDeque<(u32, u32)>,
}

impl GarbageAttackManager {
	// for mcts
	pub fn from_display(display: &Display) -> Self {
		Self {
			cm: display.cm,
			tcm: display.tcm,
			garbages: display.garbages.clone(),
		}
	}

	// for mcts
	pub fn to_info(&self) -> Vec<f32> {
		vec![
			self.cm as f32 / 10f32,
			self.tcm as f32 / 10f32,
			self.garbages
				.iter()
				.map(|(_, amt)| amt)
				.sum::<u32>() as f32,
		]
	}

	// push a new attack into pending garbage queue
	pub fn push_garbage(&mut self, width: u32, atk: u32) {
		if atk == 0 {
			return;
		}
		self.garbages.push_back((width, atk));
	}

	pub fn pop_garbage(&mut self, retain: usize) -> Vec<(u32, u32)>{
		if self.garbages.len() <= retain { return Vec::new() }
		let popsize = self.garbages.len() - retain;
		self.garbages.drain(0..popsize).collect()
	}

	// return atk
	fn counter_attack(&mut self, mut atk: u32) -> u32 {
		loop {
			// return if attack remains
			if self.garbages.is_empty() {
				break atk;
			}
			if self.garbages[0].1 >= atk {
				self.garbages[0].1 -= atk;
				if self.garbages[0].1 == 0 {
					self.garbages.pop_front();
				}
				break 0;
			}
			let popped_lines = self.garbages.pop_front().unwrap();
			atk -= popped_lines.1;
		}
	}

	pub fn calc_attack(
		&mut self,
		twist: u32,
		line_count: u32,
		code: CodeType,
		pc: bool,
	) -> (u32, u32) { // raw atk, net atk
		if line_count == 0 {
			self.cm = 0;
			if self.tcm > 0 {
				self.tcm = ATTACK_B2B_INC;
			}
			return (0, 0);
		}
		let base_atk = ATTACK_BASE[(line_count - 1) as usize];
		let twist_mult = if twist > 0 {
			ATTACK_BASE_TWIST_MULTIPLIER
				[(twist as usize - 1) * 7 + code as usize]
		} else {
			10
		};
		let mut total_mult = 10;
		total_mult += self.cm;
		self.cm += ATTACK_COMBO_INC;
		self.tcm = if twist > 0 {
			total_mult += self.tcm;
			self.tcm + ATTACK_B2B_INC
		} else {
			0
		};
		let mut atk = base_atk * twist_mult * total_mult / 1000;
		if pc {
			atk += ATTACK_PC;
		}
		(atk, self.counter_attack(atk))
	}

	pub fn read_display(&mut self, display: &Display) {
		self.garbages = display.garbages.clone();
		self.cm = display.cm;
		self.tcm = display.tcm;
	}

	pub fn write_display(&self, mut display: &mut Display) {
		display.garbages = self.garbages.clone();
		display.cm = self.cm;
		display.tcm = self.tcm;
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn test_counter_attack() {
		let mut gaman: GarbageAttackManager = Default::default();
		gaman.garbages = VecDeque::from(vec![(1, 1), (1, 2), (1, 3), (1, 4), (1, 5)]);
		gaman.counter_attack(5);
		let expect_garbage = vec![(1, 1), (1, 4), (1, 5)];
		assert!(gaman
			.garbages
			.iter()
			.zip(expect_garbage.iter())
			.fold(true, |result, (ref x, ref y)| { result & (x == y) }))
	}
}
