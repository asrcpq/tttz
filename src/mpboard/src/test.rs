use crate::board::Board;
use tttz_ruleset::CodeType;
use tttz_protocol::Piece;

pub fn generate_solidlines(heights: [usize; 10]) -> Board {
	let mut board: Board = Default::default();
	for (i, &height) in heights.iter().enumerate() {
		for j in 0..height {
			board.field[j][i] = b'i';
		}
	}
	board
}

pub fn oracle(mut board: &mut Board, floating_code: CodeType, bag: &[CodeType]) {
	board.floating_block = Piece::new(floating_code);
	board.calc_shadow();
	board.rg.bag.drain(..);
	board.rg.bag.extend(bag.iter());
}
