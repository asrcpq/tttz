use std::fmt;
use tttz_protocol::BoardReply;

#[derive(Default)]
pub struct ReplayCounter {
	atk: u32,
	piece: u32,
	key: u32,
	twist: u32,
	non_twist: u32,
	twist_flag: bool,
	duration: u128,
}

impl fmt::Display for ReplayCounter {
	fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
		let a = self.atk as f32;
		let p = self.piece as f32;
		let k = self.key as f32;
		let d = self.duration as f32;
		write!(
			formatter,
			"APP: {} KPP: {} PPS: {} TNT: {}",
			a / p,
			k / p,
			p / d * 1e6f32,
			self.twist as f32 / self.non_twist as f32,
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
				self.twist_flag = false;
			}
			BoardReply::ClearDrop(_lc, _atk, raw) => {
				self.atk += raw;
				self.piece += 1;
				self.key += 1;
				// this is not precise(consider a harddrop after twist)
				if self.twist_flag {
					self.twist += 1;
					self.twist_flag = false;
				} else {
					self.non_twist += 1;
				}
			}
			BoardReply::Ok => {
				self.key += 1;
				self.twist_flag = false;
			}
			BoardReply::BadMove => {
				self.key += 1;
			}
			BoardReply::RotateTwist => {
				self.key += 1;
				self.twist_flag = true;
			}
			_ => {}
		}
	}
}
