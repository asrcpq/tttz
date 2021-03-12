use tttz_protocol::Display;
use tttz_protocol::KeyType;

use crate::Thinker;
use cold_clear::Interface;
use libtetris::PieceMovement;

use std::collections::VecDeque;

pub struct CCBot {
	interface: Interface,
	preview_list: [u8; 6],
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
			preview_list: [7u8; 6],
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
		self.preview_list = [7u8; 6];
	}

	// TODO: handle garbages
	fn main_think(&mut self, display: Display) -> VecDeque<KeyType> {
		self.update_preview(&display.bag_preview, display.floating_block[2]);
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

impl CCBot {
	fn update_preview(&mut self, new_list: &[u8; 6], current: u8) {
		if self.preview_list[0] == 7 {
			// feed previews
			self.interface.add_next_piece(code_to_piece(current));
			for &code in new_list.iter() {
				eprintln!("add {}", code);
				self.interface.add_next_piece(code_to_piece(code));
			}
			self.preview_list = *new_list;
		} else {
			// the head of new preview is index of last preview
			'a: for last_pos in 0..6 {
				let mut old_id = last_pos;
				let mut new_pos = 0;
				loop {
					if self.preview_list[old_id] != new_list[new_pos] {
						continue 'a
					}
					new_pos += 1;
					old_id += 1;
					if old_id == 6 {
						while new_pos < 6 {
							eprintln!("add {}", new_list[new_pos]);
							self.interface.add_next_piece(code_to_piece(new_list[new_pos]));
							new_pos += 1;
						}
						break 'a
					}
				}
			}
			self.preview_list = *new_list;
		}
	}
}
