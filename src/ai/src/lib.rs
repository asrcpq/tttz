mod ai;
mod ai_utils;
pub use ai::Thinker;
mod basic_ai;
pub use basic_ai::BasicAi;
mod ccbot;
pub use ccbot::CCBot;

#[cfg(feature="MMBot")]
mod mmbot;

#[cfg(feature="MMBot")]
pub use mmbot::MMBot;

pub fn spawn_ai(algo: &str, game_type: tttz_protocol::GameType, sleep: u64)
	-> Result<std::thread::JoinHandle<()>, String>
{
	match algo {
		"basic" => {
			Ok(std::thread::spawn(move || {
				let mut basic_ai: BasicAi = Default::default();
				basic_ai.main_loop("127.0.0.1:23124", sleep, game_type);
			}))
		}
		"cc" => {
			Ok(std::thread::spawn(move || {
				let mut ccbot: CCBot = Default::default();
				ccbot.main_loop("127.0.0.1:23124", sleep, game_type);
			}))
		}
		"mm" => unimplemented!(),
		_ => {
			Err("Unknown algorithm {}".to_string())
		}
	}
}
