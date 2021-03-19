mod ai;
pub use ai::Thinker;
mod basic_ai;
pub use basic_ai::BasicAi;
mod ccbot;
pub use ccbot::CCBot;
mod mmbot;
pub use mmbot::MMBot;

pub fn spawn_ai(
	algo: &str,
	game_type: tttz_protocol::GameType,
	sleep: u64,
) -> Result<std::thread::JoinHandle<()>, String> {
	match algo {
		"basic" => Ok(std::thread::spawn(move || {
			let mut basic_ai: BasicAi = Default::default();
			basic_ai.main_loop("127.0.0.1:23124", sleep, game_type);
		})),
		"cc" => Ok(std::thread::spawn(move || {
			let mut ccbot: CCBot = Default::default();
			ccbot.main_loop("127.0.0.1:23124", sleep, game_type);
		})),
		"ccop" => Ok(std::thread::spawn(move || {
			let mut ccbot: CCBot = CCBot::new_op();
			ccbot.main_loop("127.0.0.1:23124", sleep, game_type);
		})),
		"mm" => Ok(std::thread::spawn(move || match MMBot::try_new() {
			Ok(mut mmbot) => {
				mmbot.main_loop("127.0.0.1:23124", sleep, game_type)
			}
			Err(e) => eprintln!("{:?}", e),
		})),
		_ => Err("Unknown algorithm {}".to_string()),
	}
}
