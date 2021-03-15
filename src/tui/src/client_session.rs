use termion::async_stdin;
use termion::raw::IntoRawMode;
use tttz_protocol::{ClientMsg, Display, IdType, ServerMsg};

use crate::client_display::ClientDisplay;
use crate::client_socket::ClientSocket;
use crate::keymap::TuiKey;
use crate::sound_effect::SoundEffect;
use crate::sound_manager::SoundManager;

use std::collections::HashMap;
use std::io::{stdout, Read, Write};

pub struct ClientSession {
	sound_manager: SoundManager,
	client_socket: ClientSocket,
	client_display: ClientDisplay,
	state: i32,
	id: IdType,
	mode: i32,
	textbuffer: String,
	bytebuf: Vec<u8>,
	last_display: HashMap<IdType, Display>,
	last_request_id: IdType,
}

impl ClientSession {
	pub fn new(addr: String) -> ClientSession {
		let (client_socket, id) = ClientSocket::new(&addr);
		let client_display = Default::default();
		let mut client_session = ClientSession {
			sound_manager: Default::default(),
			client_socket,
			client_display,
			state: 1,
			id,
			mode: 0,
			textbuffer: String::new(),
			bytebuf: Vec::new(),
			last_display: HashMap::new(),
			last_request_id: 0,
		};
		client_session
			.client_socket
			.socket
			.set_nonblocking(true)
			.unwrap();
		client_session.modeswitch(0);
		client_session
	}

	fn modeswitch(&mut self, new: i32) {
		self.mode = new;
		self.bytebuf.clear();
		if new == 0 {
			self.client_display.deactivate();
			self.print_prompt();
		} else {
			self.client_display.activate();
			for display in self.last_display.values() {
				self.client_display.disp_by_id(&display);
			}
		}
	}

	fn textmode_print(&self, msg: &str) {
		print!("\r{}{}\n{}", termion::clear::CurrentLine, msg, 13 as char);
		self.print_prompt();
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
			use std::str::FromStr;
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
		} else {
			self.textmode_print(&msg);
		}
	}

	fn print_prompt(&self) {
		print!("{}[36m> [0m", 13 as char);
	}

	// handle recv without display
	// true = exit
	// early return to prevent message shown
	fn handle_msg(&mut self, msg: ServerMsg) -> bool {
		match msg {
			ServerMsg::Terminate => return true,
			ServerMsg::Start(id) => {
				self.setpanel(0, self.id);
				self.setpanel(1, id);
				self.modeswitch(1);
				self.state = 2;
			}
			ServerMsg::Attack(id, amount) => {
				if let Some(mut display) = self.last_display.remove(&id) {
					display.garbages.push_back(amount);
					if self.mode == 1 {
						self.client_display.disp_atk_by_id(&display);
					}
					self.last_display.insert(id, display);
				}
			}
			ServerMsg::GameOver(_) => {
				self.state = 1;
			}
			ServerMsg::ClientList(_) => {}
			ServerMsg::Request(id) => {
				self.last_request_id = id;
			}
			_ => self.show_msg("Unknown message received!"),
		}
		self.show_msg(&msg.to_string());
		false
	}

	fn byte_handle(&mut self, byte: u8) -> bool {
		// mode == 0
		if self.mode == 0 {
			if !self.bytebuf.is_empty() {
				if byte == b'[' {
					return false;
				}
				self.bytebuf.clear();
				return false;
			}
			match byte {
				b'' => {
					// esc
					self.bytebuf.push(byte);
				}
				23 => {
					// ctrl-w
					while let Some(ch) = self.textbuffer.pop() {
						print!("{} {}", 8 as char, 8 as char);
						if ch.is_whitespace() {
							break;
						}
					}
				}
				3 => {
					// ctrl-c
					self.textbuffer = String::new();
				}
				4 => {
					// eof
					self.textbuffer = String::new();
					self.modeswitch(1);
				}
				127 => {
					// bs
					self.textbuffer.pop();
					print!("{} {}", 8 as char, 8 as char);
				}
				b'\r' => {
					// carriage return
					print!("\n\r");
					if self.proc_line(&self.textbuffer.clone()) {
						return true;
					}
					self.textbuffer = String::new();
					// when proc_line switch mode, prompt should not be printed
					if self.mode == 0 {
						self.print_prompt();
					}
				}
				byte => {
					let byte = byte as char;
					self.textbuffer.push(byte);
					print!("{}", byte);
				}
			}
			return false;
		}

		// mode == 1
		if !self.bytebuf.is_empty() {
			self.bytebuf.push(byte);
			if byte == b'[' {
				return false;
			}
		} else if byte == b'' {
			self.bytebuf.push(b'');
			return false;
		} else {
			self.bytebuf.push(byte);
		}
		// consume bytebuf
		self.handle_bytebuf()
	}

	// true = exit
	fn handle_bytebuf(&mut self) -> bool {
		match TuiKey::from_bytestring(&self.bytebuf) {
			TuiKey::ServerKey(key_event) => {
				// only send keyevent to server when playing
				if self.state == 2 {
					self.client_socket
						.send(ClientMsg::KeyEvent(key_event))
						.unwrap();
				}
			}
			TuiKey::Quit => return true,
			TuiKey::Accept => {
				self.client_socket
					.send(ClientMsg::Accept(self.last_request_id))
					.unwrap();
			}
			TuiKey::Restart => {
				if self.state == 2 {
					self.client_socket.send(ClientMsg::Suicide).unwrap();
					self.state = 3;
				} else {
					self.client_socket.send(ClientMsg::Restart).unwrap();
					self.state = 3;
				}
			}
			TuiKey::Modeswitch => self.modeswitch(0),
			TuiKey::Invalid => {}
		}
		self.bytebuf.clear();
		false
	}

	// true = exit
	fn recv_phase(&mut self) -> bool {
		if let Ok(server_msg) = self.client_socket.recv() {
			match server_msg {
				ServerMsg::Display(display) => {
					let id = display.id;
					if self.last_display.remove(&id).is_some() {
						if self.mode == 1 {
							self.client_display.disp_by_id(&display);
							self.sound_manager.play(
								&SoundEffect::from_board_reply(
									&display.board_reply,
								),
							);
						}
						self.last_display.insert(id, display);
					} else {
						eprintln!("Received display of unknown id {}", id);
					}
				}
				x => {
					if self.handle_msg(x) {
						return true;
					}
				}
			}
			stdout().flush().unwrap();
		}
		false
	}

	pub fn main_loop(&mut self) {
		let mut stdin = async_stdin().bytes();
		let stdout = stdout();
		let mut stdout = stdout.lock().into_raw_mode().unwrap();
		stdout.flush().unwrap();
		'main_loop: loop {
			if self.mode == 1 {
				self.client_display.set_offset();
			}
			if self.recv_phase() {
				break;
			}
			let mut input_flag = false;
			while let Some(Ok(byte)) = stdin.next() {
				input_flag = true;
				if self.byte_handle(byte) {
					break 'main_loop;
				}
			}
			if input_flag {
				stdout.flush().unwrap();
			}
			std::thread::sleep(std::time::Duration::from_millis(10));
		}
	}
}
