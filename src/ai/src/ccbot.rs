use tttz_protocol::Display;
use tttz_protocol::KeyType;

use crate::Thinker;
use cold_clear::Interface;
use libtetris::PieceMovement;

use std::collections::VecDeque;

pub struct CCBot {
	interface: Interface,
	first_preview: bool,
}

fn map_key(pm: PieceMovement) -> KeyType {
	match pm {
		PieceMovement::Left => KeyType::Left,
		PieceMovement::Right => KeyType::Right,
		PieceMovement::Cw => KeyType::Rotate,
		PieceMovement::Ccw => KeyType::RotateReverse,
		PieceMovement::SonicDrop => KeyType::SoftDrop,
	}
}

fn proc_moves(hold: bool, inputs: &[PieceMovement]) -> VecDeque<KeyType> {
	let mut ret = VecDeque::new();
	if hold {
		ret.push_back(KeyType::Hold);
	}
	for &input in inputs.iter() {
		ret.push_back(map_key(input));
	}
	ret.push_back(KeyType::HardDrop);
	ret
}

fn get_if() -> Interface {
	let evaluator: cold_clear::evaluation::Standard = Default::default();
	Interface::launch(
		libtetris::Board::new(),
		Default::default(),
		evaluator,
		None,
	)
}

impl Default for CCBot {
	fn default() -> CCBot {
		let interface = get_if();
		CCBot {
			interface,
			first_preview: true,
		}
	}
}

fn code_to_piece(code: u8) -> libtetris::Piece {
	use libtetris::Piece::*;
	match code {
		0 => I,
		1 => J,
		2 => L,
		3 => O,
		4 => S,
		5 => T,
		6 => Z,
		_ => { panic!("Empty is not allowed"); }
	}
}

impl Thinker for CCBot {
	fn reset(&mut self) {
		self.interface = get_if();
		self.first_preview = true;
	}

	// currently we do not deal with garbages
	fn main_think(&mut self, display: Display) -> VecDeque<KeyType> {
		if self.first_preview {
			// feed previews
			self.interface.add_next_piece(code_to_piece(display.floating_block[2]));
			for &code in display.bag_preview.iter() {
				self.interface.add_next_piece(code_to_piece(code));
			}
			self.first_preview = false;
		} else {
			self.interface.add_next_piece(code_to_piece(display.bag_preview[5]));
		}
		self.interface.request_next_move(0);
		match self.interface.block_next_move() {
			None => panic!("CC returns none!"),
			Some((moves, _info)) => {
				eprintln!("{:?}", moves.inputs);
				proc_moves(moves.hold, &moves.inputs)
			},
		}
	}
}
