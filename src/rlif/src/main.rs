use tttz_libai::*;
use tttz_protocol::{ClientMsg, ServerMsg};
use tttz_libclient::ClientSocket;
use tttz_mpboard::Game;

const ADDR: &'static str = "127.0.0.1:23124";

struct Round {
	game: Game,
}

impl Default for Round {
	fn default() -> Round {
		Round {
			game: Game::new(0, 1, Vec::new().into_iter()),
		}
	}
}

fn main() {
	let stdin = std::io::stdin();
	let mut input = String::new();
	let round: Round = Default::default();
	while let Ok(_len) = stdin.read_line(&mut input) {
		let words: Vec<&str> = input.split_whitespace().collect();
		match words[0] {
			"getNextState" => {
				let mut az_board = [[0; 20]; 20];
				let mut iter = words[1].chars();
				for i in 0..20 {
					for j in 0..20 {
						az_board[i][j] = iter.next().unwrap().to_digit(10).unwrap();
					}
				}
			}
			_ => {},
		}
	}
}
