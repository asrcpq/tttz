use once_cell::sync::Lazy;

use tttz_protocol::{KeyType, Piece};
use tttz_ruleset::*;

use std::collections::VecDeque;

pub struct GenerateKeyParam {
	pub hold_swap: bool,
	pub code: CodeType,
	pub rotation: i8,
	pub post_key: KeyType,
	pub dx: PosType,
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

// standard rotation pos
// each line is for a type of block, 4 pairs of pos(left up) indicates 4 directions
// each pos is the difference to first pair
const SRP: [[(PosType, PosType); 4]; 7] = [
	[(0, 0), (2, -2), (0, -1), (1, -2)],
	[(0, 0), (1, -1), (0, -1), (0, -1)],
	[(0, 0), (1, -1), (0, -1), (0, -1)],
	[(0, 0), (0, 0), (0, 0), (0, 0)],
	[(0, 0), (1, -1), (0, -1), (0, -1)],
	[(0, 0), (1, -1), (0, -1), (0, -1)],
	[(0, 0), (1, -1), (0, -1), (0, -1)],
];

// mass center height
pub const BLOCK_MCH: [[f32; 4]; 7] = [
	[0.0, 1.5, 0.0, 1.5],
	[0.25, 1.25, 0.75, 0.75],
	[0.25, 0.75, 0.75, 1.25],
	[0.5, 0.5, 0.5, 0.5],
	[0.5, 1.0, 0.5, 1.0],
	[0.25, 1.0, 0.75, 1.0],
	[0.5, 1.0, 0.5, 1.0],
];

pub fn generate_keys(gkp: GenerateKeyParam) -> VecDeque<KeyType> {
	let mut ret = VecDeque::new();
	if gkp.hold_swap {
		ret.push_back(KeyType::Hold);
	}
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
		ret.push_back(keycode);
	}
	if gkp.post_key != KeyType::Nothing {
		ret.push_back(KeyType::SonicDrop);
		ret.push_back(gkp.post_key);
	}
	ret
}

// return a list of possible drop pos
pub fn convolve_height(
	heights: &[PosType],
	code: CodeType,
	rot: i8,
) -> (Vec<(PosType, PosType)>, [PosType; 4], [PosType; 4]) {
	let mut ret = Vec::new();
	let mut dx = 0;
	let mut posx = [0; 4];
	let mut posy = [0; 4];
	for block in 0..4usize {
		let tmp = BPT[code as usize][rot as usize][block];
		posx[block] = tmp.0;
		posy[block] = tmp.1;
	}
	loop {
		if dx + BLOCK_WIDTH[code as usize][rot as usize] > 10 {
			break (ret, posx, posy);
		}

		let mut highest = 0;
		for block in 0..4usize {
			let height =
				heights[dx as usize + posx[block] as usize] - posy[block];
			if height > highest {
				highest = height;
			}
		}
		ret.push((dx, highest));
		dx += 1;
	}
}

pub fn get_height_and_hole(
	color: &Vec<[u8; 10]>,
) -> ([PosType; 10], PosType, usize) {
	// calc height
	let mut heights: [PosType; 10] = [0; 10];
	let mut highest_hole = 0;
	let mut highest_hole_x: PosType = -1;
	for i in 0..10 {
		let mut j: usize = color.len() - 1;
		let mut state = 0;
		loop {
			if color[j][i] == b' ' {
				if state == 1 {
					break;
				}
			} else if state == 0 {
				state = 1;
				heights[i as usize] = j as PosType + 1;
			}
			if j == 0 {
				break;
			}
			j -= 1;
		}
		if j > highest_hole {
			highest_hole = j;
			highest_hole_x = i as PosType;
		}
	}
	(heights, highest_hole_x, highest_hole)
}

pub fn count_hover_x(color: &Vec<[u8; 10]>, piece: &Piece) -> i32 {
	let hover_check: Lazy<[[Vec<(PosType, PosType)>; 4]; 7]> =
		Lazy::new(|| {
			[
				[
					vec![(0, -1), (1, -1), (2, -1), (3, -1)],
					vec![(0, -1)],
					vec![(0, -1), (1, -1), (2, -1), (3, -1)],
					vec![(0, -1)],
				],
				[
					vec![(0, -1), (1, -1), (2, -1)],
					vec![(0, -1), (1, 1)],
					vec![(0, 0), (1, 0), (2, -1)],
					vec![(0, -1), (1, -1)],
				],
				[
					vec![(0, -1), (1, -1), (2, -1)],
					vec![(0, -1), (1, -1)],
					vec![(0, -1), (1, 0), (2, 0)],
					vec![(0, 1), (1, -1)],
				],
				[
					vec![(0, -1), (1, -1)],
					vec![(0, -1), (1, -1)],
					vec![(0, -1), (1, -1)],
					vec![(0, -1), (1, -1)],
				],
				[
					vec![(0, -1), (1, -1), (2, 0)],
					vec![(0, 0), (1, -1)],
					vec![(0, -1), (1, -1), (2, 0)],
					vec![(0, 0), (1, -1)],
				],
				[
					vec![(0, -1), (1, -1), (2, -1)],
					vec![(0, -1), (1, 0)],
					vec![(0, 0), (1, -1), (2, -0)],
					vec![(0, 0), (1, -1)],
				],
				[
					vec![(0, 0), (1, -1), (2, -1)],
					vec![(0, -1), (1, 0)],
					vec![(0, 0), (1, -1), (2, -1)],
					vec![(0, -1), (1, 0)],
				],
			]
		});
	let mut hole: i32 = 0;
	for pos in hover_check[piece.code as usize][piece.rotation as usize].iter()
	{
		let x = piece.pos.0 + pos.0;
		let y = piece.pos.1 + pos.1;
		if y < 0 {
			continue;
		}
		if color[y as usize][x as usize] == b' ' {
			hole += 1;
		}
	}
	hole
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn test_mch() {
		let mut mch: [[f32; 4]; 7] = Default::default();
		for code in 0..7 {
			for rot in 0..4 {
				mch[code][rot] = BPT[code][rot]
					.iter()
					.fold(0f32, |max, data| max + data.1 as f32)
					/ 4.0;
			}
		}
		assert_eq!(mch, BLOCK_MCH);
	}
}
