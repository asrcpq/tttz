use rodio::source::Buffered;
use rodio::{Decoder, Sink, Source};

use crate::sound_effect::SoundEffect;

use std::collections::HashMap;
use std::io::{BufReader, Cursor};
use std::sync::mpsc::{channel, Receiver, Sender};

type Sound = Buffered<Decoder<BufReader<Cursor<Vec<u8>>>>>;
type SoundMap = HashMap<SoundEffect, Sound>;

struct Mixer {
	playing: Vec<Sound>,
	received: Receiver<Sound>,
}

impl Iterator for Mixer {
	type Item = i16;
	fn next(&mut self) -> Option<Self::Item> {
		self.playing.extend(self.received.try_iter());
		let mut amp: Self::Item = 0;
		let mut new_list = Vec::new();
		for mut sound in self.playing.drain(..) {
			if let Some(v) = sound.next() {
				amp = amp.saturating_add(v);
				new_list.push(sound);
			}
		}
		self.playing = new_list;
		Some(amp)
	}
}

impl Source for Mixer {
	fn current_frame_len(&self) -> Option<usize> {
		None
	}

	fn channels(&self) -> u16 {
		2
	}

	fn sample_rate(&self) -> u32 {
		44100
	}

	fn total_duration(&self) -> Option<std::time::Duration> {
		None
	}
}

fn soundmap_init() -> SoundMap {
	let mut soundmap: SoundMap = HashMap::new();
	macro_rules! load_se {
		($prefix: expr, $mapped_se: expr) => {
			// windows build, change '/' to '\'
			let sound =
				include_bytes!(concat!(env!("OUT_DIR"), "/", $prefix, ".wav"))
					.to_vec();
			soundmap.insert(
				$mapped_se,
				Decoder::new(BufReader::new(Cursor::new(sound)))
					.unwrap()
					.buffered(),
			);
		};
	}
	load_se!("soft_drop", SoundEffect::SonicDrop);
	load_se!("plain_drop", SoundEffect::PlainDrop);
	load_se!("clear_drop", SoundEffect::ClearDrop);
	load_se!("attack_drop", SoundEffect::AttackDrop);
	load_se!("attack_drop_2", SoundEffect::AttackDrop2);
	load_se!("rotate_fail", SoundEffect::Rotate(0));
	load_se!("rotate_regular", SoundEffect::Rotate(1));
	load_se!("rotate_twist", SoundEffect::Rotate(2));
	soundmap
}

pub struct SoundManager {
	soundmap: SoundMap,
	send: Sender<Sound>,
}

impl Default for SoundManager {
	fn default() -> SoundManager {
		let (send, recv) = channel();
		std::thread::spawn(move || {
			let (_stream, stream_handle) =
				rodio::OutputStream::try_default().unwrap();
			let sink = Sink::try_new(&stream_handle).unwrap();
			sink.append(Mixer {
				playing: Vec::new(),
				received: recv,
			});

			sink.sleep_until_end(); // forever
		});
		SoundManager {
			soundmap: soundmap_init(),
			send,
		}
	}
}

impl SoundManager {
	pub fn play(&mut self, se: &SoundEffect) {
		if let Some(buf) = self.soundmap.get(se) {
			let _result = self.send.send(buf.clone());
		}
	}
}
