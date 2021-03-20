mod ai;
use ai::Thinker;
mod basic_ai;
use basic_ai::BasicAi;
mod ccbot;
use ccbot::CCBot;
mod mmbot;
use mmbot::MMBot;
mod sbai;
use sbai::SBAi;

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
		"sbai" => Ok(std::thread::spawn(move || {
			let mut sbai: SBAi = Default::default();
			sbai.main_loop("127.0.0.1:23124", sleep, game_type);
		})),
		"cc" => Ok(std::thread::spawn(move || {
			let mut ccbot: CCBot = Default::default();
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
