use tttz_ai::{BasicAi, CCBot, MMBot, SBAi};
use tttz_libai::Thinker;
use tttz_mpboard::Game;

fn simulation() -> bool {
	let mut game = Game::new(1, 2, vec![].iter());
	let mut basic_ai: SBAi = Default::default();
	// let mut sbai: CCBot = Default::default();
	let mut sbai: MMBot = MMBot::try_new().unwrap();
	let mut turn = 0;
	let result = 'sim: loop {
		turn += 1;
		let keyseq = basic_ai.main_think(vec![game.generate_display(0, 0)]);
		for key_type in keyseq.into_iter() {
			let ret = game.process_key(1, 0, key_type).0;
			if ret > 0 {
				if ret == 1 {
					break 'sim true;
				}
				break 'sim false;
			}
		}
		let keyseq = sbai.main_think(vec![game.generate_display(1, 0)]);
		for key_type in keyseq.into_iter() {
			let ret = game.process_key(2, 0, key_type).0;
			if ret > 0 {
				if ret == 1 {
					break 'sim true;
				}
				break 'sim false;
			}
		}
	};
	eprintln!("End@turn {}", turn);
	result
}

fn main() {
	let mut p1win = 0;
	let total = 100;
	for i in 0..total {
		let result = simulation();
		if result {
			p1win += 1;
		}
		eprintln!("Round {}: {}", i, result);
	}
	println!("Result: p1 win {} out of {}", p1win, total);
}
