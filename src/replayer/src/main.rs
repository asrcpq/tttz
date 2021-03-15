use termion::raw::IntoRawMode;

use tttz_mpboard::{Board, Replay};
use tttz_libclient::ClientDisplay;

use std::io::{stdout, Write};

fn main() {
	let mut iter = std::env::args();
	iter.next();
	let path = iter.next().unwrap();
	let content = std::fs::read(path).unwrap();
	let replay: Replay = bincode::deserialize(&content).unwrap();

	let stdout = stdout();
	let mut stdout = stdout.lock().into_raw_mode().unwrap();

	let mut board: Board = Default::default();
	tttz_mpboard::utils::oracle(&mut board, 7, &replay.block_seq);
	let mut client_display: ClientDisplay = Default::default();
	client_display.activate();
	let mut last_time = 0;
	for msg in replay.data.iter() {
		std::thread::sleep(std::time::Duration::from_micros((msg.0 - last_time) as u64));
		last_time = msg.0;
		let board_reply = board.handle_msg(msg.1.clone());
		let display = board.generate_display(0, board_reply);
		client_display.disp_by_panel(&display, 0);
		stdout.flush().unwrap();
	}
	std::thread::sleep(std::time::Duration::from_millis(3000));
}
