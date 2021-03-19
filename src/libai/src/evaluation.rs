use tttz_protocol::{Display, Piece};
use tttz_ruleset::*;
use crate::utils::*;

pub trait Evaluator {
	fn evaluate_piece(&self, display: &Display, piece: &Piece) -> f32;
	fn evaluate_field(display: &Display) -> Self;
}

pub struct SimpleEvaluator {
	highest_hole_x: PosType,
}

impl Evaluator for SimpleEvaluator {
	fn evaluate_field(display: &Display) -> Self {
		let (_heights, highest_hole_x, _highest_hole) =
			get_height_and_hole(&display.color);
		SimpleEvaluator {
			highest_hole_x,
		}
	}

	fn evaluate_piece(&self, display: &Display, piece: &Piece) -> f32 {
		// how bad it is to put a block on the highest hole
		const COVER_WEIGHT: f32 = 0.5;
		// how bad it is to create a new hole
		const HOLE_WEIGHT: f32 = 1.0;
		// how bad it is to increase height
		const HEIGHT_WEIGHT: f32 = 1.0;

		let option_code = piece.code;
		let hole = count_hover_x(&display.color, &piece);
		let cover = (piece.pos.0 <= self.highest_hole_x
			&& piece.pos.0 + BLOCK_WIDTH[option_code as usize][piece.rotation as usize]
				> self.highest_hole_x) as PosType;
		let score = (piece.pos.1 as f32
			+ BLOCK_MCH[option_code as usize][piece.rotation as usize])
			* HEIGHT_WEIGHT + hole as f32
			* HOLE_WEIGHT + cover as f32
			* COVER_WEIGHT;
		score
	}
}
