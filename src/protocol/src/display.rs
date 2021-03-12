use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

pub const BOARD_WIDTH: usize = 10;

// interface between server and client
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Display {
	pub id: i32,
	pub color: Vec<[u8; BOARD_WIDTH]>,
	pub shadow_block: [u8; 4], // posx, posy, code, rotation
	pub floating_block: [u8; 4],
	pub hold: u8,
	pub bag_preview: [u8; 6],
	pub cm: u32,
	pub tcm: u32,
	pub garbages: VecDeque<u32>,
	pub garbage_flush: bool,
}

impl Display {
	pub fn new(id: i32) -> Display {
		Display {
			id,
			color: vec![[7; BOARD_WIDTH]; 20],
			shadow_block: [0; 4],
			floating_block: [0; 4],
			hold: 7,
			bag_preview: [7; 6],
			cm: 0,
			tcm: 0,
			garbages: VecDeque::new(),
			garbage_flush: false,
		}
	}
}
