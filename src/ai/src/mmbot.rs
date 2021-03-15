use tttz_protocol::Display;
use tttz_protocol::KeyType;

use crate::Thinker;

use std::collections::VecDeque;
use std::io::{BufRead, BufReader, Write};
use std::process::{ChildStdin, ChildStdout, Stdio};

pub struct MMBot {
	child_in: ChildStdin,
	child_out: BufReader<ChildStdout>,
}

// 5: MOV_D, USING_MOV_D should be disabled in compilation
// 12: MOV_REFRESH, // this is never used in mm
fn map_key(key: char) -> KeyType {
	match key {
		'n' => KeyType::Nothing,
		'h' => KeyType::Left,
		'l' => KeyType::Right,
		'H' => KeyType::LLeft,
		'L' => KeyType::RRight,
		'j' => KeyType::SonicDrop,
		'z' => KeyType::RotateReverse,
		'x' => KeyType::Rotate,
		'k' => KeyType::HardDrop,
		's' => KeyType::Hold,
		'd' => KeyType::RotateFlip,
		_ => unimplemented!("Unimplemented key {}", key),
	}
}

fn parse_moves(string: String) -> VecDeque<KeyType> {
	let mut ret = VecDeque::new();
	for word in string.split_whitespace() {
		let number = word.parse::<char>().expect("Parse stdout failed!");
		ret.push_back(map_key(number))
	}
	ret
}

fn convert_field(field: &[[u8; 10]]) -> String {
	let height = 20;
	let mut ret = Vec::new();
	for i in (0..height).rev() {
		let mut row = [" "; 10];
		for j in 0..10 {
			row[9 - j] = if field[i][j] == b' ' { "0" } else { "2" };
		}
		ret.push(row.join(","));
	}
	ret.join(";")
}

// mirrored
const ID_TO_CHAR_MM: [char; 7] = ['I', 'L', 'J', 'O', 'Z', 'T', 'S'];

impl Thinker for MMBot {
	fn reset(&mut self) {
		eprintln!("MMBot: Reset");
		self.write_msg("update game round 1\n");
	}

	// TODO: handle garbages
	fn main_think(&mut self, display: Display) -> VecDeque<KeyType> {
		self.write_msg(&format!(
			"update game this_piece_type {}\n",
			ID_TO_CHAR_MM[display.floating_block[2] as usize],
		));
		self.write_msg(&format!(
			"update game next_pieces {}\n",
			display
				.bag_preview
				.iter()
				.map(|&x| String::from(ID_TO_CHAR_MM[x as usize]))
				.collect::<Vec<String>>()
				.join(",")
		));
		let garbage_sum: u32 = display.garbages.iter().sum();
		self.write_msg(&format!("update bot1 inAtt {}\n", garbage_sum));
		let field_string = convert_field(&display.color);
		self.write_msg(&format!("update bot1 field {}\n", field_string,));
		self.write_msg("action2 moves 10000\n");
		let mut buf = String::new();
		self.child_out.read_line(&mut buf).unwrap();
		eprintln!("mmm {}", buf);
		parse_moves(buf)
	}
}

impl MMBot {
	pub fn try_new() -> Result<MMBot, Box<dyn std::error::Error>> {
		let mut process = std::process::Command::new(
			match std::env::var("TTTZ_MMBOT_PATH") {
				Ok(string) => string,
				Err(_) => "tttz_mmbot".to_string(),
			},
		)
		.stdin(Stdio::piped())
		.stdout(Stdio::piped())
		.spawn()?;
		let child_in = process.stdin.take().unwrap();
		let child_out = BufReader::new(process.stdout.take().unwrap());
		Ok(MMBot {
			child_in,
			child_out,
		})
	}

	pub fn write_msg(&mut self, string: &str) {
		self.child_in.write_all(string.as_bytes()).unwrap();
		eprint!("mmm {}", string);
	}
}
