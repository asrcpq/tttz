use tttz_libai::Thinker;
mod basic_ai;
pub use basic_ai::BasicAi;
// mod ccbot;
// pub use ccbot::CCBot;
mod mmbot;
pub use mmbot::MMBot;
mod sbai;
pub use sbai::SBAi;
mod stupid_mcts;

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
		"3tz" => Ok(std::thread::spawn(move || {
			let _ = std::process::Command::new(
				match std::env::var("TTTZ_TTTZBOT_PATH") {
					Ok(string) => string,
					Err(_) => "tttz_tttzbot".to_string(),
				},
			)
			.output()
			.unwrap();
		})),
		"3tz2" => Ok(std::thread::spawn(move || {
			let _ = std::process::Command::new(
				match std::env::var("TTTZ_TTTZBOT_PATH") {
					Ok(string) => string,
					Err(_) => "tttz_tttzbot".to_string(),
				},
			)
			.args(&["mode", "strategy_initiator"])
			.output()
			.unwrap();
		})),
		// "cc" => Ok(std::thread::spawn(move || {
		// 	let mut ccbot: CCBot = Default::default();
		// 	ccbot.main_loop("127.0.0.1:23124", sleep, game_type);
		// })),
		// "cc2" => Ok(std::thread::spawn(move || {
		// 	let mut ccbot = CCBot::from_eval(
		// 		serde_json::from_str(
		// 			&std::fs::read_to_string(
		// 				"thirdparty/cold-clear/optimizer/best.json",
		// 			)
		// 			.unwrap(),
		// 		)
		// 		.unwrap(),
		// 	);
		// 	ccbot.main_loop("127.0.0.1:23124", sleep, game_type);
		// })),
		"mm" => Ok(std::thread::spawn(move || match MMBot::try_new() {
			Ok(mut mmbot) => {
				mmbot.main_loop("127.0.0.1:23124", sleep, game_type)
			}
			Err(e) => eprintln!("{:?}", e),
		})),
		_ => Err("Unknown algorithm {}".to_string()),
	}
}
