extern crate ears;
use ears::{Sound, AudioController};
extern crate tttz_protocol;
use tttz_protocol::SoundEffect;

use std::collections::HashMap;

fn soundmap_init() -> HashMap<SoundEffect, Sound> {
	let mut soundmap = HashMap::new();
	soundmap.insert(
		SoundEffect::PlainDrop,
		Sound::new("./resources/se/plaindrop.wav").expect("error"),
	);
	for i in 1..=20 {
		soundmap.insert(
			SoundEffect::ClearDrop(i),
			Sound::new(&format!("./resources/se/cleardrop{}.wav", i)).expect("error"),
		);
		soundmap.insert(
			SoundEffect::AttackDrop(i),
			Sound::new(&format!("./resources/se/attackdrop{}.wav", i)).expect("error"),
		);
	}
	soundmap
}

pub struct SoundManager {
	soundmap: HashMap<SoundEffect, Sound>,
}

impl Default for SoundManager {
	fn default() -> SoundManager {
		SoundManager {
			soundmap: soundmap_init(),
		}
	}
}

impl SoundManager {
	pub fn play(&mut self, se: SoundEffect) {
		if let Some(snd2) = self.soundmap.get_mut(&se) {
			snd2.play();
		}
	}
}
