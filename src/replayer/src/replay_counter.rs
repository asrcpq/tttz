use tttz_protocol::BoardReply;
use std::fmt;

#[derive(Default)]
pub struct ReplayCounter {
	atk: u32,
	piece: u32,
	key: u32,
	duration: u128,
}

impl fmt::Display for ReplayCounter {
	fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
		let a = self.atk as f64;
		let p = self.piece as f64;
		let k = self.key as f64;
		let d = self.duration as f64;
		write!(
			formatter,
			"APP: {} KPP: {} PPS: {}",
			a / p,
			k / p,
			p / d * 1e6f64,
		)
	}
}

impl ReplayCounter {
	// TODO: write test
	pub fn count(&mut self, br: &BoardReply, t: u128) {
		self.duration = t;
		match br {
			BoardReply::PlainDrop(_) => {
				self.piece += 1;
				self.key += 1;
			},
			BoardReply::ClearDrop(_, atk2) => {
				self.atk += atk2;
				self.piece += 1;
				self.key += 1;
			},
			BoardReply::Ok => {
				self.key += 1;
			},
			BoardReply::BadMove => {
				self.key += 1;
			},
			BoardReply::RotateTwist => {
				self.key += 1;
			},
			_ => {},
		}
	}
}
