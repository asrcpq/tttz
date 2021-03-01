extern crate termion;
use std::io::{stdout, Read, Write};
use termion::async_stdin;
use termion::raw::IntoRawMode;
extern crate bincode;

extern crate rustyline;
use rustyline::Editor;

mod client_display;
use client_display::ClientDisplay;
mod client_socket;
use client_socket::ClientSocket;

extern crate mpboard;
use mpboard::display::Display;

struct GameState {
	state: i32,
}

impl Default for GameState {
	fn default() -> GameState {
		GameState {
			state: 1,
		}
	}
}

fn game_mode(
	client_socket: &ClientSocket,
	client_display: &mut ClientDisplay,
	game_state: &mut GameState,
) -> i32 {
	client_display.activate();
	client_socket.socket.set_nonblocking(true).unwrap();
	let stdout = stdout();
	let mut stdout = stdout.lock().into_raw_mode().unwrap();
	let mut stdin = async_stdin().bytes();
	let mut buf = [0; 1024];
	loop {
		if let Ok(amt) = client_socket.recv(&mut buf) {
			// all long messages are board display
			client_display.set_offset();
			if amt < 16 {
				let msg = std::str::from_utf8(&buf[..amt]).unwrap();
				if msg.starts_with("sigatk ") {
					let pending_atk = msg[7..amt].parse::<u32>().unwrap();
					client_display.disp_atk_pub(pending_atk, 0);
				} else if msg == "die" || msg == "win" {
					game_state.state = 1;
				}
				client_display.disp_msg(&msg);
				continue;
			} else {
				let decoded: Display =
					bincode::deserialize(&buf[..amt]).unwrap();
				client_display.disp_by_id(&decoded);
			}
			stdout.flush().unwrap();
		}
		if let Some(Ok(byte)) = stdin.next() {
			match byte {
				b'q' => {
					break;
				}
				b'r' => {
					if game_state.state == 2 {
						client_socket.send(b"suicide").unwrap();
						game_state.state = 3;
					} else {
						client_socket.send(b"pair").unwrap();
						game_state.state = 3;
					}
				}
				b'/' => {
					client_display.deactivate();
					return 0;
				}
				_ => {
					if game_state.state == 2 {
						client_socket
							.send(
								format!("key {}", byte as char).as_bytes(),
							)
							.unwrap();
					}
				}
			}
		}
		std::thread::sleep(std::time::Duration::from_millis(10));
	}
	return 2;
}

fn proc_line(
	line: String,
	client_socket: &ClientSocket,
	client_display: &mut ClientDisplay,
) {
	let split: Vec<&str> = line.split_whitespace().collect();
	if split[0] == "msg" {
		client_socket.send(&line.bytes().collect::<Vec<u8>>()[4..]).unwrap();
	} else if split[0] == "panel" {
		if split.len() < 3 {
			return
		}
		let panel = match split[1].parse::<usize>() {
			Ok(id) => id,
			Err(_) => return,
		};
		let id = match split[2].parse::<i32>() {
			Ok(id) => id,
			Err(_) => return,
		};
		client_display.setpanel(panel, id);
	} else {
		if line == "pair" {
			client_socket.socket.set_nonblocking(false).unwrap();
		}
		client_socket.send(line.as_bytes()).unwrap();
	}
}

fn text_mode(
	client_socket: &ClientSocket,
	client_display: &mut ClientDisplay,
	game_state: &mut GameState,
	id: i32
) -> i32 {
	let mut rl = Editor::<()>::new();
	client_socket.socket.set_nonblocking(true).unwrap();
	loop {
		let readline = rl.readline("> ");
		match readline {
			Ok(line) => {
				if line.trim().is_empty() {
					return 1;
				}
				rl.add_history_entry(line.as_str());
				proc_line(line, client_socket, client_display);
			},
			Err(_) => {
				return 2;
			}
		}
		let mut buf = [0; 1024];
		// drain the data
		while let Ok(amt) = client_socket.recv(&mut buf) {
			if amt > 16 {
				continue
			}
			let msg = String::from(
				std::str::from_utf8(&buf[..amt]).unwrap()
			);
			let split = msg
				.split_whitespace()
				.collect::<Vec<&str>>();
			if split[0] == "startvs" {
				let opid = split[1].parse::<i32>().unwrap();
				client_display.setpanel(0, id);
				client_display.setpanel(1, opid);
				game_state.state = 2;
				return 1;
			}
			println!("{}", msg);
			break
		}
	}
}

fn main() {
	let mut mode = 0; // 0: text, 1: game
	let mut iter = std::env::args();
	let mut game_state: GameState = Default::default();
	iter.next();
	let addr = match iter.next() {
		Some(string) => string,
		None => "127.0.0.1:23124".to_string(),
	};
	let (client_socket, id) = ClientSocket::new(&addr);
	let mut client_display = ClientDisplay::new(id);
	loop {
		mode = match mode {
			0 => text_mode(
				&client_socket,
				&mut client_display,
				&mut game_state,
				id,
			),
			1 => game_mode(
				&client_socket,
				&mut client_display,
				&mut game_state,
			),
			2 => break,
			_ => unreachable!(),
		};
	}
}
