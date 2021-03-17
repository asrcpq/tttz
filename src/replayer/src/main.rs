use std::io::{stdout, Read, Write};
use termion::async_stdin;
use termion::raw::IntoRawMode;
use tttz_libclient::ClientDisplay;
mod replay_simulator;
use replay_simulator::{ReplaySimulator, SeekResult};
mod replay_counter;

fn main() {
	let mut rss = Vec::new();
	let mut iter = std::env::args();
	iter.next();
	let mut spd = 1.0;
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
			_ => {}
		}
	}

	// termion and display works inside
	{
		let mut stdin = async_stdin().bytes();
		let stdout = stdout();
		let mut stdout = stdout.lock().into_raw_mode().unwrap();

		let mut client_display: ClientDisplay = Default::default();
		client_display.setpanel(0, 1);
		client_display.setpanel(1, 2);
		client_display.activate();

		let replay_start_time = std::time::Instant::now();
		let mut elapsed;
		'main_loop: loop {
			elapsed =
				(replay_start_time.elapsed().as_micros() as f64 * spd) as u128;
			let mut all_end = true;
			for rs in rss.iter_mut() {
				match rs.seek_forward(elapsed) {
					SeekResult::End => {}
					SeekResult::Ok(None) => all_end = false,
					SeekResult::Ok(Some(display)) => {
						client_display.disp_by_id(&display);
						all_end = false;
					}
				}
			}
			if all_end {
				break 'main_loop;
			}
			stdout.flush().unwrap();
			while let Some(Ok(byte)) = stdin.next() {
				if byte == b'q' {
					break 'main_loop;
				}
			}
			std::thread::sleep(std::time::Duration::from_millis(10));
		}
		client_display.deactivate();
	}

	for rs in rss.iter() {
		rs.print_rc();
	}
}
