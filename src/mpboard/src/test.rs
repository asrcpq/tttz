use crate::block::Block;
use crate::board::Board;

pub fn generate_solidlines(heights: [usize; 10]) -> Board {
	let mut board: Board = Default::default();
	for (i, &height) in heights.iter().enumerate() {
		for j in 0..height {
			board.field[j][i] = b'i';
		}
	}
	board
}

pub fn oracle(mut board: &mut Board, floating_code: u8, bag: &[u8]) {
	board.floating_block = Block::new(floating_code);
	board.calc_shadow();
	board.rg.bag.drain(..);
	board.rg.bag.extend(bag.iter());
}
