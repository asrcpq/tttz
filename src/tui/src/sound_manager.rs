extern crate ears;
use ears::{Sound, AudioController};
extern crate tttz_protocol;
use tttz_protocol::SoundEffect;

use std::collections::HashMap;

fn soundmap_init() -> HashMap<SoundEffect, Sound> {
	let mut soundmap = HashMap::new();
	if let Ok(sound) = Sound::new("./resources/se/plaindrop.wav") {
		soundmap.insert(SoundEffect::PlainDrop, sound);
	}
	for i in 1..=20 {
		if let Ok(sound) = Sound::new(&format!("./resources/se/cleardrop{}.wav", i)) {
			soundmap.insert(SoundEffect::ClearDrop(i), sound);
		}
		if let Ok(sound) = Sound::new(&format!("./resources/se/attackdrop{}.wav", i)) {
			soundmap.insert(SoundEffect::AttackDrop(i), sound);
		}
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
