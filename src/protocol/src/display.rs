use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

pub const BOARD_WIDTH: usize = 10;

// interface between server and client
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Display {
	pub id: i32,
	pub color: Vec<[u8; BOARD_WIDTH]>,
	pub shadow_block: [u8; 4], // posx, posy, code, rotation
	pub tmp_block: [u8; 4],
	pub hold: u8,
	pub bag_preview: [u8; 6],
	pub combo_multiplier: u32,
	pub b2b_multiplier: u32,
	pub garbages: VecDeque<u32>,
}

impl Display {
	pub fn new(id: i32) -> Display {
		Display {
			id,
			color: vec![[7; BOARD_WIDTH]; 20],
			shadow_block: [0; 4],
			tmp_block: [0; 4],
			hold: 7,
			bag_preview: [7; 6],
			combo_multiplier: 0,
			b2b_multiplier: 0,
			garbages: VecDeque::new(),
		}
	}

	pub fn generate_solidlines(heights: [usize; 10]) -> Display {
		let mut display = Display::new(1);
		for i in 0..10 {
			for j in 0..heights[i] {
				display.color[39 - j][i] = 1;
			}
		}
		display
	}
}
