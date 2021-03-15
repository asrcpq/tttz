use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use tttz_protocol::BoardMsg;
use tttz_ruleset::CodeType;

use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Replay {
	pub start_time: SystemTime,
	pub data: Vec<(u128, BoardMsg)>,
	pub block_seq: Vec<CodeType>,
}

impl Default for Replay {
	fn default() -> Replay {
		Replay {
			start_time: SystemTime::now(),
			data: Vec::new(),
			block_seq: Vec::new(),
		}
	}
}

impl Replay {
	pub fn push_block(&mut self, code: CodeType) {
		self.block_seq.push(code);
	}

	pub fn push_operation(&mut self, board_msg: BoardMsg) {
		let since_the_epoch = SystemTime::now()
			.duration_since(self.start_time)
			.expect("Time went backwards");
		self.data.push((since_the_epoch.as_micros(), board_msg));
	}

	pub fn save(
		&self,
		filename: &str,
	) -> Result<bool, Box<dyn std::error::Error>> {
		if let Some(proj_dirs) = ProjectDirs::from("", "asrcpq", "tttz") {
			let path = proj_dirs.data_dir().join("replay");
			std::fs::create_dir_all(&path)?;
			let path = path.join(&format!(
				"{}-{}.tttz_replay",
				self.start_time
					.duration_since(UNIX_EPOCH)
					.expect("Time went backwards")
					.as_secs(),
				filename,
			));
			eprintln!("Writing replay to {:?}", path);
			let mut file = std::fs::File::create(path)?;
			let data = &bincode::serialize(self)?;
			file.write_all(data)?;
			return Ok(true);
		}
		Ok(false)
	}
}
