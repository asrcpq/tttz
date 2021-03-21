// stupid ai, put block to make least holes and lowest height
use crate::ai::Thinker;
use tttz_libai::{access_floodfill, route_solver};
use tttz_libai::evaluation::{Evaluator, SimpleEvaluator};
use tttz_protocol::{Display, KeyType, Piece};

use std::collections::VecDeque;

#[derive(Default)]
pub struct BasicAi {}

impl Thinker for BasicAi {
	fn reset(&mut self) {}

	fn main_think(&mut self, mut display: Display) -> VecDeque<KeyType> {
		if display.hold == 7 {
			let mut ret = VecDeque::new();
			ret.push_back(KeyType::Hold);
			display.hold = display.floating_block.code;
			display.floating_block.code = display.bag_preview[0];
		}

		let simple_evaluator = SimpleEvaluator::evaluate_field(&display.color);
		let mut best_score = f32::NEG_INFINITY;
		let mut best_piece = Piece::new(0);
		let mut best_id = 0;
		for (id, &option_code) in [display.floating_block.code, display.hold]
			.iter()
			.enumerate()
		{
			for piece in access_floodfill(&display.color, option_code).iter() {
				let score = simple_evaluator.evaluate_piece(&display.color, piece).0;
				if score > best_score {
					best_score = score;
					best_piece = piece.clone();
					best_id = id;
				}
			}
		}
		let mut key_seq = VecDeque::new();
		if best_id == 1 {
			key_seq.push_back(KeyType::Hold);
		}
		key_seq.extend(route_solver(&display.color, &best_piece).unwrap_or(
			VecDeque::new()
		));
		key_seq.push_back(KeyType::HardDrop);
		key_seq
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use tttz_protocol::BoardReply;

	#[test]
	fn test_obvious_zspin() {
		let mut color = vec![[b'i'; 10]; 2];
		color.extend(vec![[b' '; 10]; 18]);
		color[1][0] = b' ';
		color[1][1] = b' ';
		color[0][1] = b' ';
		color[0][2] = b' ';
		let display = Display {
			seq: 0,
			id: 0,
			color,
			shadow_block: Piece::new(0),
			floating_block: Piece::new(6),
			hold: 0,
			bag_preview: [0; 6],
			cm: 0,
			tcm: 0,
			garbages: Default::default(),
			board_reply: BoardReply::Ok,
		};
		let mut ai: BasicAi = Default::default();
		let mut ret = ai.main_think(display);
		eprintln!("ret {:?}", ret);
		let key = ret.pop_back().unwrap();
		assert_eq!(key, KeyType::HardDrop);
		let key = ret.pop_back().unwrap();
		assert!(key != KeyType::SonicDrop);
	}
}
