use crate::sound_manager::SoundManager;
use crate::sound_effect::SoundEffect;
use tttz_protocol::{Display, IdType, BoardMsg, KeyType};
use tttz_mpboard::Board;

pub struct ClientRenderer {
	gamekey_history: Vec<KeyType>,
	// disruptive_checkpoints: Vec<u32>,
	crb: Board, // client render board
	id: IdType,
}

impl ClientRenderer {
	pub fn new(id: IdType) -> ClientRenderer{
		ClientRenderer {
			gamekey_history: Vec::new(),
			crb: Default::default(),
			id,
		}
	}

	pub fn reset(&mut self) {
		self.crb = Default::default();
		self.gamekey_history.clear();
	}

	pub fn push_key(&mut self, key_type: KeyType, sm: &SoundManager) -> Display {
		self.gamekey_history.push(key_type);
		let rep = self.crb.handle_msg(BoardMsg::KeyEvent(key_type));
		sm.play(&SoundEffect::from_board_reply(&rep));
		self.crb.generate_display(self.id, rep)
	}

	pub fn backtrack(&mut self, seq: u32, display: &mut Display) {
		self.crb.update_from_display(&display);
		(seq as usize..self.gamekey_history.len()).map(|id| {
			// self.show_msg(&format!("redo id {} seq {}", id, seq));
			self.crb.handle_msg(
				BoardMsg::KeyEvent(self.gamekey_history[id])
			)
		}).last()
		.map(|rep| 
			*display = self.crb.generate_display(self.id, rep)
		);
	}

	pub fn get_seq(&self) -> u32 {
		self.gamekey_history.len() as u32
	}
}
