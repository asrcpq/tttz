use crate::board::Board;

pub fn generate_solidlines(heights: [usize; 10]) -> Board {
	let mut board = Board::new(0);
	for i in 0..10 {
		for j in 0..heights[i] {
			board.color[39 - j][i] = 1;
		}
	}
	board
}
