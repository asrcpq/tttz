use tttz_protocol::{BoardReply, Display, KeyType};
use tttz_ruleset::CodeType;

use crate::Thinker;
use cold_clear::Interface;
use cold_clear::evaluation::Standard;
use libtetris::PieceMovement;

use std::collections::VecDeque;

pub struct CCBot {
	pub interface: Interface,
	pub preview_list: [CodeType; 6],
	pub evaluator: Standard,
}

fn map_key(pm: PieceMovement) -> KeyType {
	match pm {
		PieceMovement::Left => KeyType::Left,
		PieceMovement::Right => KeyType::Right,
		PieceMovement::Cw => KeyType::Rotate,
		PieceMovement::Ccw => KeyType::RotateReverse,
		PieceMovement::SonicDrop => KeyType::SonicDrop,
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

fn get_if(eval: Standard) -> Interface {
	Interface::launch(
		libtetris::Board::new(),
		Default::default(),
		eval,
		None,
	)
}

impl Default for CCBot {
	fn default() -> CCBot {
		let evaluator: cold_clear::evaluation::Standard = Default::default();
		let interface = get_if(evaluator.clone());
		CCBot {
			interface,
			preview_list: [7; 6],
			evaluator,
		}
	}
}

fn code_to_piece(code: CodeType) -> libtetris::Piece {
	use libtetris::Piece::*;
	match code {
		0 => I,
		1 => J,
		2 => L,
		3 => O,
		4 => S,
		5 => T,
		6 => Z,
		_ => {
			panic!("Empty is not allowed");
		}
	}
}

impl Thinker for CCBot {
	fn reset(&mut self) {
		eprintln!("CCBOT: Reset");
		self.interface = get_if(self.evaluator.clone());
		self.preview_list = [7; 6];
	}

	// TODO: handle garbages
	fn main_think(&mut self, display: Display) -> VecDeque<KeyType> {
		self.update_preview(&display.bag_preview, display.floating_block.code);
		if match display.board_reply {
			BoardReply::GarbageOverflow(x) if x > 0 => true,
			BoardReply::PlainDrop(x) if x > 0 => true,
			_ => false,
		} {
			let mut field = [[false; 10]; 40];
			for (row, each_row) in field.iter_mut().take(20).enumerate() {
				for (col, color) in each_row.iter_mut().enumerate() {
					*color = display.color[row][col] != b' ';
				}
			}
			self.interface.reset(field, display.tcm > 0, display.cm / 3);
		}
		// eprintln!("{:?}", display);
		let garbage_sum = display.garbages
			.iter()
			.map(|x| x.1)
			.sum();
		self.interface.request_next_move(garbage_sum);
		std::thread::sleep(std::time::Duration::from_millis(10));
		match self.interface.block_next_move() {
			None => return VecDeque::new(),
			Some((moves, _info)) => {
				// eprintln!("{:?}", moves.inputs);
				proc_moves(moves.hold, &moves.inputs)
			}
		}
	}
}

impl CCBot {
	pub fn from_eval(eval: Standard) -> CCBot {
		CCBot {
			interface: get_if(eval.clone()),
			preview_list: [7; 6],
			evaluator: eval,
		}
	}

	fn update_preview(&mut self, new_list: &[CodeType; 6], current: CodeType) {
		if self.preview_list[0] == 7 {
			// feed previews
			self.interface.add_next_piece(code_to_piece(current));
			for &code in new_list.iter() {
				// eprintln!("add {}", code);
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
						continue 'a;
					}
					new_pos += 1;
					old_id += 1;
					if old_id == 6 {
						while new_pos < 6 {
							// eprintln!("add {}", new_list[new_pos]);
							self.interface.add_next_piece(code_to_piece(
								new_list[new_pos],
							));
							new_pos += 1;
						}
						break 'a;
					}
				}
			}
			self.preview_list = *new_list;
		}
	}
}
