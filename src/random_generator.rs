use rand::seq::SliceRandom;
use rand::thread_rng;
use rand::rngs::ThreadRng;
use std::collections::VecDeque;

pub struct RandomGenerator {
	rng: ThreadRng,
	bag: VecDeque<u8>,
}

impl Default for RandomGenerator {
	fn default() -> RandomGenerator {
		let mut rg = RandomGenerator {
			rng: thread_rng(),
			bag: VecDeque::new(),
		};
		rg.generate_bag();
		return rg;
	}
}

impl RandomGenerator {
	fn generate_bag(&mut self) {
		let mut b = vec![0, 1, 2, 3, 4, 5, 6];
		b.shuffle(&mut self.rng);
		self.bag.extend(b.into_iter());
	}

	pub fn get(&mut self) -> u8 {
		if self.bag.len() <= 7 {
			self.generate_bag();
		}
		self.bag.pop_front().unwrap()
	}
}
