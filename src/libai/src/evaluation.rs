use tttz_protocol::{Display, Piece};
use tttz_ruleset::*;
use crate::utils::*;

pub trait Evaluator {
	fn evaluate_piece(&self, display: &Display, piece: &Piece) -> f32;
	fn evaluate_field(display: &Display) -> Self;
}

pub struct SimpleEvaluator {
	// how bad it is to put a block on the highest hole
	pub cover_weight: f32,
	// how bad it is to create a new hole
	pub hole_weight: f32,
	// how bad it is to increase height
	pub height_weight: f32,

	highest_hole_x: PosType,
}

impl Evaluator for SimpleEvaluator {
	fn evaluate_field(display: &Display) -> Self {
		let (_heights, highest_hole_x, _highest_hole) =
			get_height_and_hole(&display.color);
		SimpleEvaluator {
			highest_hole_x,
			cover_weight: 0.5,
			hole_weight: 1.0,
			height_weight: 1.0,
		}
	}

	fn evaluate_piece(&self, display: &Display, piece: &Piece) -> f32 {
		let option_code = piece.code;
		let hole = count_hover_x(&display.color, &piece);
		let cover = (piece.pos.0 <= self.highest_hole_x
			&& piece.pos.0 + BLOCK_WIDTH[option_code as usize][piece.rotation as usize]
				> self.highest_hole_x) as PosType;
		let score = (piece.pos.1 as f32
			+ BLOCK_MCH[option_code as usize][piece.rotation as usize])
			* self.height_weight + hole as f32
			* self.hole_weight + cover as f32
			* self.cover_weight;
		score
	}
}
