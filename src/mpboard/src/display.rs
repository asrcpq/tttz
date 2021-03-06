extern crate serde;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

// interface between server and client
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Display {
	pub id: i32,
	pub color: Vec<u8>,
	pub shadow_pos: [u8; 8],
	pub shadow_code: u8,
	pub tmp_pos: [u8; 8],
	pub tmp_code: u8,
	pub hold: u8,
	pub bag_preview: [u8; 6],
	pub combo: u32,
	pub b2b: bool,
	pub garbages: VecDeque<u32>,
}

impl Display {
	pub fn new(id: i32) -> Display {
		Display {
			id,
			color: vec![7; 10 * 40],
			shadow_pos: [0; 8],
			shadow_code: 0,
			tmp_pos: [0; 8],
			tmp_code: 0,
			hold: 7,
			bag_preview: [0; 6],
			combo: 0,
			b2b: false,
			garbages: VecDeque::new(),
		}
	}
}
