use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;
use rand::thread_rng;
use tttz_ruleset::CodeType;
use std::collections::VecDeque;

#[derive(Clone)]
pub struct RandomGenerator {
	pub rng: ThreadRng, // directly called for garbage generation
	pub bag: VecDeque<CodeType>,
}

impl Default for RandomGenerator {
	fn default() -> RandomGenerator {
		let mut rg = RandomGenerator {
			rng: thread_rng(),
			bag: VecDeque::new(),
		};
		rg.generate_bag();
		rg
	}
}

impl RandomGenerator {
	fn generate_bag(&mut self) {
		let mut b = vec![0, 1, 2, 3, 4, 5, 6];
		// let mut b = vec![0; 7]; // for debug
		b.shuffle(&mut self.rng);
		self.bag.extend(b.into_iter());
	}

	pub fn get(&mut self) -> CodeType {
		if self.bag.len() <= 7 {
			self.generate_bag();
		}
		self.bag.pop_front().unwrap()
	}
}
