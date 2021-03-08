extern crate rodio;
use rodio::source::Buffered;
use rodio::{Decoder, OutputStream, Sink, Source};
extern crate tttz_protocol;
use tttz_protocol::SoundEffect;

use std::collections::HashMap;
use std::io::{BufReader, Cursor};

type SoundMap = HashMap<SoundEffect, Buffered<Decoder<BufReader<Cursor<Vec<u8>>>>>>;

fn soundmap_init() -> SoundMap {
	let mut soundmap: SoundMap = HashMap::new();
	macro_rules! load_se {
		($prefix: expr, $mapped_se: expr) => {
			// TODO: windows build
			let sound = include_bytes!(concat!("se/", $prefix, ".wav")).to_vec();
			soundmap.insert($mapped_se,
				Decoder::new(BufReader::new(Cursor::new(sound)))
				.unwrap()
				.buffered()
			);
		}
	}
	load_se!("plaindrop", SoundEffect::PlainDrop);
	load_se!("cleardrop", SoundEffect::ClearDrop);
	load_se!("attackdrop", SoundEffect::AttackDrop);
	soundmap
}

// should not drop stream or no sound
#[allow(dead_code)]
pub struct SoundManager {
	sink: Sink,
	stream: OutputStream,
	soundmap: SoundMap,
}

impl Default for SoundManager {
	fn default() -> SoundManager {
		let (stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
		let sink = Sink::try_new(&stream_handle).unwrap();
		SoundManager {
			sink, 
			stream,
			soundmap: soundmap_init(),
		}
	}
}

impl SoundManager {
	pub fn play(&self, se: SoundEffect) {
		if let Some(buf) = self.soundmap.get(&se) {
			self.sink.append(buf.clone());
		}
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn test_sound() {
		let sm: SoundManager = Default::default();
		sm.play(SoundEffect::ClearDrop);
		std::thread::sleep(std::time::Duration::from_millis(200));
	}
}
