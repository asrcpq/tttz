use crate::utils::*;
use tttz_mpboard::Field;
use tttz_protocol::Piece;
use tttz_ruleset::*;

pub trait Evaluator {
	fn evaluate_piece(
		&self,
		color: &Vec<[u8; 10]>,
		piece: &Piece,
	) -> (f32, Field);
	fn evaluate_field(display: &Vec<[u8; 10]>) -> Self;
}

#[derive(Debug)]
pub struct SimpleEvaluator {
	highest_hole_x: PosType,
}

impl Evaluator for SimpleEvaluator {
	fn evaluate_field(color: &Vec<[u8; 10]>) -> Self {
		let (_heights, highest_hole_x, _highest_hole) =
			get_height_and_hole(color);
		SimpleEvaluator { highest_hole_x }
	}

	fn evaluate_piece(
		&self,
		color: &Vec<[u8; 10]>,
		piece: &Piece,
	) -> (f32, Field) {
		// how bad it is to put a block on the highest hole
		const COVER_WEIGHT: f32 = 0.5;
		// how bad it is to create a new hole
		const HOLE_WEIGHT: f32 = 0.5;
		// how bad it is to increase height
		const HEIGHT_WEIGHT: f32 = 1.0;

		let option_code = piece.code;
		let hole = count_hover_x(color, &piece);
		let cover = (piece.pos.0 <= self.highest_hole_x
			&& piece.pos.0
				+ BLOCK_WIDTH[option_code as usize][piece.rotation as usize]
				> self.highest_hole_x) as PosType;
		let score = (piece.pos.1 as f32
			+ BLOCK_MCH[option_code as usize][piece.rotation as usize])
			* HEIGHT_WEIGHT
			+ hole as f32 * HOLE_WEIGHT
			+ cover as f32 * COVER_WEIGHT;

		let mut new_field = Field::from_color(color);
		let twist = new_field.test_twist(&mut piece.clone());
		let lc = new_field.settle_block(&piece);
		// do not handle combo
		let atk = (twist as f32 + 1.0) * (lc as f32 - 0.5); // simple approx
		let q = (atk * 4. - score) / 10.0 + 1.0;
		(q, new_field)
	}
}
