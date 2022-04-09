use crate::{Board, Game};
use tttz_protocol::Piece;
use tttz_ruleset::{CodeType, PosType};

pub fn generate_solidlines(heights: [usize; 10]) -> Board {
	let mut board: Board = Default::default();
	for (i, &height) in heights.iter().enumerate() {
		for j in 0..height {
			board.field[j][i] = b'i';
		}
	}
	board
}

pub fn oracle(
	mut board: &mut Board,
	floating_code: CodeType,
	bag: &[CodeType],
) {
	board.rg.bag.drain(..);
	board.rg.bag.extend(bag.iter());
	board.rg.bag_id = 0;
	if floating_code != 7 {
		board.floating_block = Piece::new(floating_code);
	} else {
		board.floating_block.code = 7;
		board.spawn_block();
	}
	board.calc_shadow();
}

pub fn oracle_garbage(board: &mut Board, shift: &[f32], slot: &[PosType]) {
	board.rg.shift.drain(..);
	board.rg.shift.extend(shift.iter());
	board.rg.slots.drain(..);
	board.rg.slots.extend(slot.iter());
}

// for ML: concat boards
#[allow(clippy::needless_range_loop)]
pub fn color_manipulation(game: &mut Game, color: [[u8; 20]; 20]) {
	for i in 0..20 {
		for j in 0..10 {
			game.boards[0].field[i][j] = color[i][j];
		}
	}
	for i in 0..20 {
		for j in 0..10 {
			game.boards[1].field[i][j] = color[i][j + 10];
		}
	}
}
