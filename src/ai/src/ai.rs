use tttz_libclient::ClientSocket;
use tttz_protocol::{ClientMsg, Display, GameType, KeyType, ServerMsg};

use std::collections::VecDeque;

pub trait Thinker {
	fn reset(&mut self);

	fn main_think(&mut self, display: Display) -> VecDeque<KeyType>;

	fn main_loop(
		&mut self,
		addr: &str,
		sleep_millis: u64,
		game_type: GameType,
	) {
		let strategy = game_type != GameType::Speed;
		let (client_socket, id) = ClientSocket::new(&addr);
		let main_sleep = 10;

		let mut state = 3;
		let mut last_display: Option<Display> = None;
		let mut moveflag = false;
		let mut operation_queue: VecDeque<KeyType> = VecDeque::new();
		loop {
			std::thread::sleep(std::time::Duration::from_millis(main_sleep));
			// read until last screen
			while let Ok(server_msg) = client_socket.recv() {
				match server_msg {
					ServerMsg::Display(display) => {
						if display.id == id {
							last_display = Some(display);
						} else {
							// strategy ai moves after user move
							if strategy {
								moveflag = true;
							}
						}
					}
					ServerMsg::GameOver(_) => {
						self.reset();
						state = 1;
					}
					ServerMsg::Start(_) => {
						state = 2;
					}
					ServerMsg::Request(id) => {
						if state != 2 {
							client_socket.send(ClientMsg::Accept(id)).unwrap();
						}
					}
					ServerMsg::Invite(id) => {
						if state != 2 {
							client_socket.send(ClientMsg::Request(id)).unwrap();
						}
					}
					ServerMsg::Terminate => {
						return;
					}
					_ => eprintln!("Skipping msg: {}", server_msg),
				}
			}
			if strategy {
				if state == 2 && moveflag {
					if let Some(decoded) = last_display.take() {
						if operation_queue.is_empty() {
							operation_queue = self.main_think(decoded);
						}
						client_socket
							.send(ClientMsg::KeyEvent(
								operation_queue.pop_front().unwrap(),
							))
							.unwrap();
						moveflag = false;
					}
				}
			} else if let Some(decoded) = last_display.take() {
				if state == 2 {
					operation_queue = self.main_think(decoded);
					while let Some(key_type) = operation_queue.pop_front() {
						client_socket
							.send(ClientMsg::KeyEvent(key_type))
							.unwrap();
						std::thread::sleep(std::time::Duration::from_millis(
							sleep_millis,
						));
					}
				}
				last_display = None;
			}
		}
	}
}
