extern crate termion;
use std::io::{stdout, Read, Write};
use termion::async_stdin;
use termion::raw::IntoRawMode;
extern crate bincode;

use crate::client_display::ClientDisplay;
use crate::client_socket::ClientSocket;

extern crate mypuzzle_mpboard;
use mypuzzle_mpboard::display::Display;

use std::collections::HashMap;

pub struct ClientSession {
	client_socket: ClientSocket,
	client_display: ClientDisplay,
	state: i32,
	id: i32,
	mode: i32,
	textbuffer: String,
	last_display: HashMap<i32, Display>,
}

impl ClientSession {
	pub fn new(addr: String) -> ClientSession {
		let (client_socket, id) = ClientSocket::new(&addr);
		let client_display = Default::default();
		ClientSession {
			client_socket,
			client_display,
			state: 1,
			id,
			mode: 0,
			textbuffer: String::new(),
			last_display: HashMap::new(),
		}
	}

	fn modeswitch(&mut self, new: i32) {
		self.mode = new;
		if new == 0 {
			self.client_display.deactivate();
		} else {
			self.client_display.activate();
		}
	}

	fn textmode_print(&self, msg: &str) {
		print!(
			"{}{}{}{}{}{}",
			termion::cursor::Hide,
			termion::cursor::Goto(1, 2),
			termion::clear::CurrentLine,
			msg,
			termion::cursor::Show,
			termion::cursor::Goto(1, 1),
		);
	}

	// true quit
	pub fn proc_line(&mut self, line: &str) -> bool {
		let split: Vec<&str> = line.split_whitespace().collect();
		if split.is_empty() {
			self.modeswitch(1);
			return false;
		}
		if split[0] == "quit" {
			return true;
		} else if split[0] == "sleep" {
			// for scripts
			if let Ok(t) = split[1].parse::<u64>() {
				std::thread::sleep(std::time::Duration::from_millis(t));
			}
		} else if split[0] == "msg" {
			self.client_socket
				.send(&line.bytes().collect::<Vec<u8>>()[4..])
				.unwrap();
		} else if split[0] == "myid" {
			self.textmode_print(&format!("{}", self.id));
		} else if split[0] == "panel" {
			if split.len() < 3 {
				return false;
			}
			let panel = match split[1].parse::<usize>() {
				Ok(id) => id,
				Err(_) => return false,
			};
			let id = match split[2].parse::<i32>() {
				Ok(id) => id,
				Err(_) => return false,
			};
			self.setpanel(panel, id);
		} else {
			self.client_socket.send(line.as_bytes()).unwrap();
		}
		false
	}

	fn setpanel(&mut self, panel: usize, id: i32) {
		self.last_display.insert(id, Display::new(id));
		self.client_display.setpanel(panel, id);
	}

	fn handle_recv(&mut self, msg: &str) -> Option<i32> {
		let split = msg.split_whitespace().collect::<Vec<&str>>();
		if split[0] == "startvs" {
			let opid = split[1].parse::<i32>().unwrap();
			self.setpanel(0, self.id);
			self.setpanel(1, opid);
			self.modeswitch(1);
			self.state = 2;
			return Some(1);
		} else if split[0] == "start" {
			self.setpanel(0, self.id);
			self.modeswitch(1);
			self.state = 2;
			return Some(1);
		} else if split[0] == "sigatk" {
			let id = split[1].parse::<i32>().unwrap();
			let pending_atk = split[2].parse::<u32>().unwrap();
			if let Some(mut display) = self.last_display.remove(&id) {
				display.garbages.push_back(pending_atk);
				self.client_display.disp_atk_by_id(&display);
				self.last_display.insert(id, display);
			}
		} else if msg == "die" || msg == "win" {
			self.state = 1;
		}
		None
	}

	fn byte_handle(&mut self, byte: u8) -> bool {
		// mode == 0
		if self.mode == 0 {
			if byte == 23 {
				while let Some(ch) = self.textbuffer.pop() {
					if ch.is_whitespace() {
						break;
					}
				}
			} else if byte == 3 {
				self.textbuffer = String::new();
			} else if byte == 127 {
				self.textbuffer.pop();
			} else if byte == b'\r' {
				if self.proc_line(&self.textbuffer.clone()) {
					return true;
				}
				self.textbuffer = String::new();
			} else {
				self.textbuffer.push(byte as char);
			}
			print!(
				"{}{}{}",
				termion::cursor::Goto(1, 1),
				termion::clear::CurrentLine,
				self.textbuffer,
			);
			return false;
		}

		// mode == 1
		match byte {
			b'q' => {
				return true;
			}
			b'r' => {
				if self.state == 2 {
					self.client_socket.send(b"suicide").unwrap();
					self.state = 3;
				} else {
					self.client_socket.send(b"restart").unwrap();
					self.state = 3;
				}
			}
			b'/' => {
				self.client_display.deactivate();
				self.modeswitch(0);
			}
			_ => {
				if self.state == 2 {
					self.client_socket
						.send(format!("key {}", byte as char).as_bytes())
						.unwrap();
				}
			}
		}
		false
	}

	pub fn main_loop(&mut self) {
		let mut stdin = async_stdin().bytes();
		let stdout = stdout();
		let mut stdout = stdout.lock().into_raw_mode().unwrap();
		self.client_socket.socket.set_nonblocking(true).unwrap();
		let mut buf = [0; 1024];
		loop {
			if let Ok(amt) = self.client_socket.recv(&mut buf) {
				// all long messages are board display
				if self.mode == 1 {
					self.client_display.set_offset();
				}
				if amt < 64 {
					let msg =
						String::from(std::str::from_utf8(&buf[..amt]).unwrap());
					self.handle_recv(&msg);
					if self.mode == 1 {
						self.client_display.disp_msg(&msg);
					} else {
						self.textmode_print(&msg);
					}
				} else {
					if self.mode == 1 {
						let display: Display =
							bincode::deserialize(&buf[..amt]).unwrap();
						if self.last_display.remove(&display.id).is_some() {
							self.client_display.disp_by_id(&display);
							self.last_display.insert(display.id, display);
						} else {
							eprintln!("Receiving unexpected id {}", display.id);
						}
					}
				}
				stdout.flush().unwrap();
			}
			if let Some(Ok(byte)) = stdin.next() {
				if self.byte_handle(byte) {
					break;
				}
				stdout.flush().unwrap();
			}
			std::thread::sleep(std::time::Duration::from_millis(10));
		}
	}
}
