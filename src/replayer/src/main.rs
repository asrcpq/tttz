use termion::async_stdin;
use termion::raw::IntoRawMode;

use tttz_mpboard::{Board, Replay};
use tttz_protocol::BoardReply;
use tttz_libclient::ClientDisplay;

use std::io::{stdout, Read, Write};

fn main() {
	let mut iter = std::env::args();
	iter.next();
	let mut path = None;
	let mut spd = 1.0;
	while let Some(string) = iter.next() { match string.as_ref() {
		"path" => {
			path = Some(iter.next().unwrap());
		},
		"speed" => {
			spd = iter.next().unwrap().parse::<f32>().unwrap();
		}
		_ => {},
	}}
	let content = std::fs::read(path.unwrap()).unwrap();
	let replay: Replay = bincode::deserialize(&content).unwrap();

	let mut stdin = async_stdin().bytes();
	let stdout = stdout();
	let mut stdout = stdout.lock().into_raw_mode().unwrap();

	let mut board: Board = Default::default();
	tttz_mpboard::utils::oracle(&mut board, 7, &replay.block_seq);
	assert!(!replay.block_seq.is_empty());
	tttz_mpboard::utils::oracle_garbage(
		&mut board,
		&replay.garbage_shift_check,
		&replay.garbage_slots,
	);
	let mut client_display: ClientDisplay = Default::default();
	client_display.setpanel(0, 1);
	client_display.activate();

	let mut atk = 0;
	let mut piece = 0;
	let mut key = 0;
	let mut iter = replay.data.iter().peekable();
	let replay_start_time = std::time::Instant::now(); 
	'main_loop: loop {
		while replay_start_time.elapsed().as_micros() > match iter.peek() {
			None => break 'main_loop,
			Some(s) => (s.0 as f32 / spd) as u128,
		} {
			let msg = iter.next().unwrap();
			let board_reply = board.handle_msg(msg.1.clone());
			match board_reply {
				BoardReply::PlainDrop(_) => {
					piece += 1;
					key += 1;
				},
				BoardReply::ClearDrop(_, atk2) => {
					atk += atk2;
					piece += 1;
					key += 1;
				},
				BoardReply::Ok => {
					key += 1;
				},
				BoardReply::BadMove => {
					key += 1;
				},
				BoardReply::RotateTwist => {
					key += 1;
				},
				_ => {},
			}
			let display = board.generate_display(1, board_reply);
			client_display.disp_by_panel(&display, 0);
			stdout.flush().unwrap();
		}
		while let Some(Ok(byte)) = stdin.next() {
			if byte == b'q' {
				break 'main_loop
			}
		}
		std::thread::sleep(std::time::Duration::from_millis(10));
	}
	std::mem::drop(client_display);
	let duration = replay.data.last().unwrap().0 as f32 / 1e6f32;
	println!(
		"PPS: {}, APP: {}, KPP: {}\n",
		piece as f32 / duration,
		atk as f32 / piece as f32,
		key as f32 / piece as f32,
	)
}
