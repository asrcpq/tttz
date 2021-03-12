use tttz_protocol::{Display, KeyType};
use tttz_ruleset::*;

use std::collections::VecDeque;

pub struct GenerateKeyParam {
	pub hold_swap: bool,
	pub code: u8,
	pub rotation: i8,
	pub post_key: KeyType,
	pub dx: i32,
}

impl Default for GenerateKeyParam {
	fn default() -> GenerateKeyParam {
		GenerateKeyParam {
			hold_swap: false,
			code: 7,
			rotation: -1,
			post_key: KeyType::Nothing,
			dx: -1,
		}
	}
}

pub fn generate_keys(gkp: GenerateKeyParam) -> VecDeque<KeyType> {
	let mut ret = VecDeque::new();
	if gkp.hold_swap { ret.push_back(KeyType::Hold); }
	let current_posx = INITIAL_POS[gkp.code as usize];
	let rotated_pos0 =
		current_posx + SRP[gkp.code as usize][gkp.rotation as usize].0;
	let (keycode, times) = if gkp.dx == 0 {
		(KeyType::LLeft, 1)
	} else if gkp.dx
		== 10 - BLOCK_WIDTH[gkp.code as usize][gkp.rotation as usize]
	{
		(KeyType::RRight, 1)
	} else if rotated_pos0 > gkp.dx {
		(KeyType::Left, rotated_pos0 - gkp.dx)
	} else {
		(KeyType::Right, gkp.dx - rotated_pos0)
	};
	if gkp.rotation == 1 {
		ret.push_back(KeyType::Rotate);
	} else if gkp.rotation == 3 {
		ret.push_back(KeyType::RotateReverse);
	} else if gkp.rotation == 2 {
		ret.push_back(KeyType::RotateFlip);
	}
	for _ in 0..times {
		ret.push_back(keycode.clone());
	}
	if gkp.post_key != KeyType::Nothing {
		ret.push_back(KeyType::SoftDrop);
		ret.push_back(gkp.post_key);
	}
	ret.push_back(KeyType::HardDrop);
	ret
}

// return a list of possible drop pos
pub fn convolve_height(heights: &[u8], code: u8, rot: i8) ->
	(Vec<(u8, u8)>, [u8; 4], [u8; 4])
{
	let mut ret = Vec::new();
	let mut dx = 0;
	let mut posx = [0; 4];
	let mut posy = [0; 4];
	for block in 0..4usize {
		let tmp = BPT[code as usize][rot as usize][block];
		posx[block] = tmp.0 as u8;
		posy[block] = tmp.1 as u8;
	}
	loop {
		if dx + BLOCK_WIDTH[code as usize][rot as usize] as u8 > 10 {
			break (ret, posx, posy)
		}

		let mut highest = 0;
		for block in 0..4usize {
			let height = heights[dx as usize + posx[block] as usize] as i32
				- posy[block] as i32
				+ 1;
			if height > highest {
				highest = height;
			}
		}
		ret.push((dx, highest as u8));
		dx += 1;
	}
}

pub fn get_height_and_hole(display: &Display) -> ([u8; 10], i32, usize) {
	// calc height
	let mut heights = [0u8; 10];
	let mut highest_hole = 0;
	let mut highest_hole_x: i32 = -1;
	for i in 0..10 {
		let mut j: usize = 19;
		let mut state = 0;
		loop {
			if display.color[j][i] == 7 {
				if state == 1 {
					break;
				}
			} else if state == 0 {
				state = 1;
				heights[i as usize] = j as u8 + 1;
			}
			if j == 0 {
				break;
			}
			j -= 1;
		}
		if j > highest_hole {
			highest_hole = j;
			highest_hole_x = i as i32;
		}
	}
	return (heights, highest_hole_x, highest_hole)
}
