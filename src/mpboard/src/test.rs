use crate::board::Board;

pub fn generate_solidlines(heights: [usize; 10]) -> Board {
	let mut board = Board::new(0);
	for (i, &height) in heights.iter().enumerate() {
		for j in 0..height {
			board.color[j][i] = 1;
		}
	}
	board
}
