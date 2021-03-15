use std::ops::{Deref, Index, IndexMut};

type Colors = Vec<[u8; 10]>;

pub struct Field {
	pub color: Colors,
}

impl Default for Field {
	fn default() -> Field {
		Field {
			color: vec![[b' '; 10]; 40],
		}
	}
}

impl Deref for Field {
	type Target = Colors;

	fn deref(&self) -> &Self::Target {
		&self.color
	}
}

impl Index<usize> for Field {
	type Output = [u8; 10];
	fn index(&self, index: usize) -> &Self::Output {
		&self.color[index]
	}
}

impl IndexMut<usize> for Field {
	fn index_mut(&mut self, index: usize) -> &mut [u8; 10] {
		&mut self.color[index]
	}
}
