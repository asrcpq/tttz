use tttz_mpboard::Game;
use tttz_ai::{BasicAi, SBAi, SBAiNext, MMBot, CCBot};
use tttz_ai::ai::Thinker;

fn simulation() -> bool {
	let mut game = Game::new(1, 2, vec![].iter());
	let mut basic_ai: BasicAi = Default::default();
	let mut sbai: SBAi = Default::default();
	// let mut sbai: CCBot = Default::default();
	// let mut sbai: MMBot = MMBot::try_new().unwrap();
	loop {
		let keyseq = basic_ai.main_think(game.generate_display(0, 0));
		for key_type in keyseq.into_iter() {
			let ret = game.process_key(1, 0, key_type).0;
			if ret > 0 {
				if ret == 1 {
					return true
				}
				return false
			}
		}
		let keyseq = sbai.main_think(game.generate_display(1, 0));
		for key_type in keyseq.into_iter() {
			let ret = game.process_key(2, 0, key_type).0;
			if ret > 0 {
				if ret == 1 {
					return true
				}
				return false
			}
		}
	}
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
