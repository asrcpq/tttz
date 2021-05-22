use std::io::{stdout, Read, Write};
use termion::async_stdin;
use termion::raw::IntoRawMode;
use tttz_libclient::{ClientDisplay, SoundEffect, SoundManager};
mod replay_simulator;
use replay_simulator::{ReplaySimulator, SeekResult};
mod replay_counter;

fn main() {
	let mut rss = Vec::new();
	let mut iter = std::env::args();
	let mut pause = false;
	iter.next();
	let mut spd = 1.0;
	let mut constant_flag = false;
	while let Some(string) = iter.next() {
		match string.as_ref() {
			"path" => {
				let path = iter.next().unwrap();
				eprintln!("Load replay from {}", path);
				rss.push(ReplaySimulator::load(rss.len() as i32 + 1, &path));
			}
			"speed" => {
				spd = iter.next().unwrap().parse::<f64>().unwrap();
			}
			"constant" => {
				constant_flag = true;
			}
			_ => {}
		}
	}

	let sm: SoundManager = Default::default();

	// termion and display works inside
	let sleep_flag = {
		let mut stdin = async_stdin().bytes();
		let stdout = stdout();
		let mut stdout = stdout.lock().into_raw_mode().unwrap();

		let mut client_display: ClientDisplay = Default::default();
		client_display.setpanel(0, 1);
		client_display.setpanel(1, 2);
		client_display.activate();

		let mut elapsed = 0;
		let sleep_flag = 'main_loop: loop {
			let mut all_end = true;
			for rs in rss.iter_mut() {
				let (end_flag, display) =
					match rs.seek_forward((elapsed as f64 * spd) as u128) {
						SeekResult::End(x) => (true, x),
						SeekResult::Ok(x) => (false, x),
					};
				all_end &= end_flag;
				if let Some(display) = display {
					client_display.set_offset();
					sm.play(&SoundEffect::from_board_reply(
						&display.board_reply,
					));
					client_display.disp_by_id(&display);
					if constant_flag {
						stdout.flush().unwrap();
						std::thread::sleep(std::time::Duration::from_millis(
							60,
						));
					}
				}
			}
			if all_end {
				break 'main_loop true;
			}
			while let Some(Ok(byte)) = stdin.next() {
				if byte == b'q' {
					break 'main_loop false;
				}
				if byte == b' ' {
					pause = !pause;
				}
			}
			if !constant_flag || pause {
				std::thread::sleep(std::time::Duration::from_millis(10));
			}
			if !pause {
				stdout.flush().unwrap();
				elapsed += 10_000;
			}
		};
		client_display.deactivate();
		sleep_flag
	};

	if sleep_flag {
		std::thread::sleep(std::time::Duration::from_millis(1000));
	}

	for rs in rss.iter() {
		rs.print_rc();
	}
}
