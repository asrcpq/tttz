use tttz_mpboard::Field;
use tttz_protocol::Piece;
use tttz_ruleset::*;
use crate::utils::*;

use std::collections::VecDeque;

pub fn access_floodfill(color: &[[u8; 10]], code: CodeType) -> Vec<Piece> {
	let heights = get_heights(color);
	let mut queue: VecDeque<Piece> = VecDeque::new();
	for rotation in 0..4 {
		for &pos in convolve_height(&heights, code, rotation).0.iter() {
			queue.push_back(Piece {
				pos,
				code,
				rotation,
			})
		}
	}
	Vec::new()
}
