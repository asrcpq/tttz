use tttz_protocol::KeyType;

pub enum TuiKey {
	Quit,
	Restart,
	Accept,
	Modeswitch,
	ServerKey(KeyType), // keys sent to server
	Invalid,
}

impl TuiKey {
	pub fn from_bytestring(keyseq: &[u8]) -> TuiKey {
		use TuiKey::*;
		match keyseq {
			b"q" => Quit,
			b"r" => Restart,
			b"a" => Accept,
			b"/" => Modeswitch,
			b"h" => ServerKey(KeyType::Left),
			b"H" => ServerKey(KeyType::LLeft),
			b"l" => ServerKey(KeyType::Right),
			b"L" => ServerKey(KeyType::RRight),
			b" " => ServerKey(KeyType::Hold),
			b"j" => ServerKey(KeyType::SonicDrop),
			b"k" => ServerKey(KeyType::HardDrop),
			b"x" => ServerKey(KeyType::Rotate),
			b"z" => ServerKey(KeyType::RotateReverse),
			b"d" => ServerKey(KeyType::RotateFlip),
			b"[D" => ServerKey(KeyType::Left),
			b"[C" => ServerKey(KeyType::Right),
			b"[A" => ServerKey(KeyType::HardDrop),
			b"[B" => ServerKey(KeyType::SonicDrop),
			_ => Invalid,
		}
	}
}
