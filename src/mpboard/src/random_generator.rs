use rand::rngs::ThreadRng;
use rand::Rng;
use rand::seq::SliceRandom;
use rand::thread_rng;
use tttz_ruleset::{CodeType, PosType};

// TODO: use trait "ReproducibleSequence"?
#[derive(Clone)]
pub struct RandomGenerator {
	rng: ThreadRng, // directly called for garbage generation
	pub(in crate) bag: Vec<CodeType>,
	pub(in crate) bag_id: usize,
	pub(in crate) slots: Vec<PosType>,
	slots_id: usize,
	pub(in crate) shift: Vec<f32>, // shift check
	shift_id: usize,
}

impl Default for RandomGenerator {
	fn default() -> RandomGenerator {
		let mut rg = RandomGenerator {
			rng: thread_rng(),
			bag: Vec::new(),
			slots: Vec::new(),
			shift: Vec::new(),
			bag_id: 0,
			slots_id: 0,
			shift_id: 0,
		};
		rg.generate_bag();
		rg.generate_slots();
		rg.generate_shift();
		rg
	}
}

impl RandomGenerator {
	fn generate_bag(&mut self) {
		let mut b = vec![0, 1, 2, 3, 4, 5, 6];
		b.shuffle(&mut self.rng);
		self.bag.extend(b.into_iter());
	}

	fn generate_slots(&mut self) {
		for _ in 0..10 {
			self.slots.push(self.rng.gen_range(0..10));
		}
	}

	fn generate_shift(&mut self) {
		for _ in 0..10 {
			self.shift.push(self.rng.gen::<f32>());
		}
	}

	pub fn get_shift(&mut self) -> f32 {
		if self.shift.len() - self.shift_id < 10 {
			self.generate_shift();
		}
		self.shift_id += 1;
		self.shift[self.shift_id - 1]
	}

	pub fn get_slot(&mut self) -> PosType {
		if self.slots.len() - self.slots_id < 10 {
			self.generate_slots()
		}
		self.slots_id += 1;
		self.slots[self.slots_id - 1]
	}

	pub fn get_code(&mut self) -> CodeType {
		eprintln!("{:?}", self.bag);
		if self.bag.len() - self.bag_id <= 7 {
			self.generate_bag();
		}
		self.bag_id += 1;
		self.bag[self.bag_id - 1]
	}

	pub fn preview_code(&self) -> [CodeType; 6] {
		let mut ret = [0; 6];
		for i in 0..6 {
			ret[i] = self.bag[self.bag_id + i];
		}
		ret
	} 
}
