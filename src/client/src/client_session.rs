extern crate termion;
use std::io::{stdout, Read, Write};
use termion::async_stdin;
use termion::raw::IntoRawMode;

use crate::client_display::ClientDisplay;
use crate::client_socket::ClientSocket;

extern crate tttz_mpboard;
use tttz_mpboard::display::Display;
extern crate tttz_protocol;
use tttz_protocol::{ClientMsg, KeyType, ServerMsg};

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
			// no need to send server quit, which is done in client_socket's drop
			return true;
		} else if split[0] == "sleep" {
			// for scripts
			if let Ok(t) = split[1].parse::<u64>() {
				std::thread::sleep(std::time::Duration::from_millis(t));
			}
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
			match ClientMsg::from_str(line) {
				Ok(client_msg) => {
					self.client_socket.send(client_msg).unwrap();
				}
				Err(_) => {
					self.show_msg("Command failed");
				}
			}
		}
		false
	}

	fn setpanel(&mut self, panel: usize, id: i32) {
		self.last_display.insert(id, Display::new(id));
		self.client_display.setpanel(panel, id);
	}

	// display msg in all modes
	fn show_msg(&self, msg: &str) {
		// show msg
		if self.mode == 1 {
			self.client_display.disp_msg(&msg);
			self.client_display.disp_msg(&msg);
		} else {
			self.textmode_print(&msg);
		}
	}

	// handle recv without display
	fn handle_msg(&mut self, msg: ServerMsg) {
		match msg {
			ServerMsg::Start(id) => {
				self.setpanel(0, self.id);
				self.setpanel(1, id);
				self.modeswitch(1);
				self.state = 2;
			},
			ServerMsg::Attack(id, amount) => {
				if let Some(mut display) = self.last_display.remove(&id) {
					display.garbages.push_back(amount);
					self.client_display.disp_atk_by_id(&display);
					self.last_display.insert(id, display);
				}
			},
			ServerMsg::GameOver(_) => {
				self.state = 1;
			}
			ServerMsg::Request(_) => {}
			_ => { eprintln!("Unknown message received!") }
		}
		self.show_msg(&msg.to_string());
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
					self.client_socket.send(ClientMsg::Suicide).unwrap();
					self.state = 3;
				} else {
					self.client_socket.send(ClientMsg::Restart).unwrap();
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
						.send(ClientMsg::KeyEvent(
							match byte {
								b'h' => {KeyType::Left},
								b'H' => {KeyType::LLeft},
								b'l' => {KeyType::Right},
								b'L' => {KeyType::RRight},
								b' ' => {KeyType::Hold},
								b'j' => {KeyType::SoftDrop},
								b'k' => {KeyType::HardDrop},
								b'J' => {KeyType::Down1},
								b'K' => {KeyType::Down5},
								b'x' => {KeyType::Rotate},
								b'z' => {KeyType::RotateReverse},
								b'd' => {KeyType::RotateFlip},
								_ => return false,
							}
						))
						.unwrap();
				}
			}
		}
		false
	}

	fn recv_phase(&mut self) {
		if let Ok(server_msg) = self.client_socket.recv() {
			match server_msg {
				ServerMsg::Display(display) => {
					if self.last_display.remove(&display.id).is_some() {
						self.client_display.disp_by_id(&display);
						self.last_display.insert(display.id, display.into_owned());
					} else {
						eprintln!("Received display of unknown id {}", display.id);
					}
				},
				x => {self.handle_msg(x) },
			}
			stdout().flush().unwrap();
			return
		}
	}

	pub fn main_loop(&mut self) {
		let mut stdin = async_stdin().bytes();
		let stdout = stdout();
		let mut stdout = stdout.lock().into_raw_mode().unwrap();
		self.client_socket.socket.set_nonblocking(true).unwrap();
		loop {
			if self.mode == 1 {
				self.client_display.set_offset();
			}
			self.recv_phase();
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
