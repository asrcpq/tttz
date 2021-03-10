use tttz_protocol::{Display, KeyType};
use tttz_ruleset::*;

use std::collections::VecDeque;

pub fn generate_keys(
	hold_swap: bool,
	code: u8,
	rotation: i8,
	post_key: KeyType,
	dx: i32,
) -> VecDeque<KeyType> {
	let mut ret = VecDeque::new();
	if hold_swap { ret.push_back(KeyType::Hold); }
	let current_posx = INITIAL_POS[code as usize];
	let rotated_pos0 =
		current_posx + SRP[code as usize][rotation as usize].0;
	let (keycode, times) = if dx == 0 {
		(KeyType::LLeft, 1)
	} else if dx
		== 10 - BLOCK_WIDTH[code as usize][rotation as usize]
	{
		(KeyType::RRight, 1)
	} else if rotated_pos0 > dx {
		(KeyType::Left, rotated_pos0 - dx)
	} else {
		(KeyType::Right, dx - rotated_pos0)
	};
	if rotation == 1 {
		ret.push_back(KeyType::Rotate);
	} else if rotation == 3 {
		ret.push_back(KeyType::RotateReverse);
	} else if rotation == 2 {
		ret.push_back(KeyType::RotateFlip);
	}
	for _ in 0..times {
		ret.push_back(keycode.clone());
	}
	if post_key != KeyType::Nothing {
		ret.push_back(post_key);
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

		let mut highest = 40;
		for block in 0..4usize {
			let height = (heights[dx as usize + posx[block] as usize]) - posy[block] - 1;
			if height < highest {
				highest = height;
			}
		}
		ret.push((dx, highest as u8));
		dx += 1;
	}
}

pub fn get_height_and_hole(display: &Display) -> ([u8; 10], i32, usize) {
	// calc height
	let mut heights = [40u8; 10];
	let mut highest_hole = 40;
	let mut highest_hole_x: i32 = -1;
	for i in 0..10 {
		let mut j: usize = 0;
		let mut state = 0;
		loop {
			if display.color[j][i] == 7 {
				if state == 1 {
					break;
				}
			} else if state == 0 {
				state = 1;
				heights[i as usize] = j as u8;
			}
			j += 1;
			if j == 40 {
				break;
			}
		}
		if j > highest_hole {
			highest_hole = j;
			highest_hole_x = i as i32;
		}
	}
	return (heights, highest_hole_x, highest_hole)
}
