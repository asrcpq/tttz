use crate::board::Board;
use crate::block::Block;

pub fn generate_solidlines(heights: [usize; 10]) -> Board {
	let mut board = Board::new(0);
	for (i, &height) in heights.iter().enumerate() {
		for j in 0..height {
			board.color[j][i] = 1;
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
