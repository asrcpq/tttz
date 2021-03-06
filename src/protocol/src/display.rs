use serde::{Deserialize, Serialize};

use tttz_ruleset::CodeType;
use crate::{BoardReply, IdType, Piece};

use std::collections::VecDeque;

pub const BOARD_WIDTH: usize = 10;

// interface between server and client
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Display {
	pub id: IdType,
	pub color: Vec<[u8; BOARD_WIDTH]>,
	pub shadow_block: Piece,
	pub floating_block: Piece,
	pub hold: CodeType,
	pub bag_preview: [CodeType; 6],
	pub cm: u32,
	pub tcm: u32,
	pub garbages: VecDeque<u32>,
	pub board_reply: BoardReply,
}

impl Display {
	pub fn new(id: IdType) -> Display {
		Display {
			id,
			color: vec![[b' '; BOARD_WIDTH]; 20],
			shadow_block: Piece::new(0),
			floating_block: Piece::new(0),
			hold: 7,
			bag_preview: [7; 6],
			cm: 0,
			tcm: 0,
			garbages: VecDeque::new(),
			board_reply: BoardReply::Ok,
		}
	}
}
