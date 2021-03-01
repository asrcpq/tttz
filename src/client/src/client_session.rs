extern crate termion;
use std::io::{stdout, Read, Write};
use termion::async_stdin;
use termion::raw::IntoRawMode;
extern crate bincode;

extern crate rustyline;
use rustyline::Editor;

use crate::client_display::ClientDisplay;
use crate::client_socket::ClientSocket;

extern crate mpboard;
use mpboard::display::Display;

pub struct ClientSession {
	client_socket: ClientSocket,
	client_display: ClientDisplay,
	state: i32,
	id: i32,
}

impl ClientSession {
	pub fn new(addr: String) -> ClientSession {
		let (client_socket, id) = ClientSocket::new(&addr);
		let client_display = ClientDisplay::new(id);
		ClientSession {
			client_socket,
			client_display,
			state: 1,
			id,
		}
	}
	fn game_mode(&mut self) -> i32 {
		self.client_display.activate();
		self.client_socket.socket.set_nonblocking(true).unwrap();
		let stdout = stdout();
		let mut stdout = stdout.lock().into_raw_mode().unwrap();
		let mut stdin = async_stdin().bytes();
		let mut buf = [0; 1024];
		loop {
			if let Ok(amt) = self.client_socket.recv(&mut buf) {
				// all long messages are board display
				self.client_display.set_offset();
				if amt < 16 {
					let msg =
						String::from(std::str::from_utf8(&buf[..amt]).unwrap());
					self.handle_recv(&msg);
					self.client_display.disp_msg(&msg);
					continue;
				} else {
					let decoded: Display =
						bincode::deserialize(&buf[..amt]).unwrap();
					self.client_display.disp_by_id(&decoded);
				}
				stdout.flush().unwrap();
			}
			if let Some(Ok(byte)) = stdin.next() {
				match byte {
					b'q' => {
						break;
					}
					b'r' => {
						if self.state == 2 {
							self.client_socket.send(b"suicide").unwrap();
							self.state = 3;
						} else {
							self.client_socket.send(b"pair").unwrap();
							self.state = 3;
						}
					}
					b'/' => {
						self.client_display.deactivate();
						return 0;
					}
					_ => {
						if self.state == 2 {
							self.client_socket
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
		2
	}

	fn proc_line(&mut self, line: &str) {
		let split: Vec<&str> = line.split_whitespace().collect();
		if split[0] == "msg" {
			self.client_socket
				.send(&line.bytes().collect::<Vec<u8>>()[4..])
				.unwrap();
		} else if split[0] == "panel" {
			if split.len() < 3 {
				return;
			}
			let panel = match split[1].parse::<usize>() {
				Ok(id) => id,
				Err(_) => return,
			};
			let id = match split[2].parse::<i32>() {
				Ok(id) => id,
				Err(_) => return,
			};
			self.client_display.setpanel(panel, id);
		} else {
			if line == "pair" {
				self.client_socket.socket.set_nonblocking(false).unwrap();
			}
			self.client_socket.send(line.as_bytes()).unwrap();
		}
	}

	fn handle_recv(&mut self, msg: &str) -> Option<i32> {
		let split = msg.split_whitespace().collect::<Vec<&str>>();
		if split[0] == "startvs" {
			let opid = split[1].parse::<i32>().unwrap();
			self.client_display.setpanel(0, self.id);
			self.client_display.setpanel(1, opid);
			self.state = 2;
			return Some(1);
		} else if split[0] == "sigatk" {
			let pending_atk = split[1].parse::<u32>().unwrap();
			self.client_display.disp_atk_pub(pending_atk, 0);
		} else if msg == "die" || msg == "win" {
			self.state = 1;
		}
		None
	}

	fn text_mode(&mut self) -> i32 {
		let mut rl = Editor::<()>::new();
		self.client_socket.socket.set_nonblocking(true).unwrap();
		loop {
			let readline = rl.readline("> ");
			match readline {
				Ok(line) => {
					if !line.trim().is_empty() {
						rl.add_history_entry(line.as_str());
						self.proc_line(&line);
					}
				}
				Err(rustyline::error::ReadlineError::Eof) => {
					return 0;
				}
				_ => {
					return 2;
				}
			}
			let mut buf = [0; 1024];
			// drain the data
			while let Ok(amt) = self.client_socket.recv(&mut buf) {
				if amt > 16 {
					continue;
				}
				let msg =
					String::from(std::str::from_utf8(&buf[..amt]).unwrap());
				if let Some(x) = self.handle_recv(&msg) {
					return x;
				}
				println!("{}", msg);
				break;
			}
		}
	}

	pub fn main_loop(&mut self) {
		let mut mode = 0; // 0: text, 1: game
		loop {
			mode = match mode {
				0 => self.text_mode(),
				1 => self.game_mode(),
				2 => break,
				_ => unreachable!(),
			};
		}
	}
}
