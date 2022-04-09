// interactive ai interface

use tttz_libclient::ClientSocket;
use tttz_protocol::{
	BoardReply, ClientMsg, Display, GameType, KeyType, ServerMsg,
};

use std::collections::VecDeque;

pub trait Thinker {
	fn reset(&mut self);

	fn main_think(&mut self, displays: Vec<Display>) -> VecDeque<KeyType>;

	fn main_loop(
		&mut self,
		addr: &str,
		sleep_millis: u64,
		game_type: GameType,
	) {
		let mut strategy = true;
		let mut moveflag = false;
		let mut main_sleep = 10;
		match game_type {
			GameType::Speed => strategy = false,
			GameType::Strategy(round_sleep, initiator) => {
				main_sleep = round_sleep;
				moveflag = initiator;
			}
			_ => unreachable!(),
		}
		let (client_socket, id) = ClientSocket::new(addr);

		let mut state = 3;
		let mut last_display: [Option<Display>; 2] = [None, None];
		let mut operation_queue: VecDeque<KeyType>;
		loop {
			std::thread::sleep(std::time::Duration::from_millis(main_sleep));
			// read until last screen
			while let Ok(server_msg) = client_socket.recv() {
				match server_msg {
					ServerMsg::Display(display) => {
						if display.id == id {
							last_display[0] = Some(display);
						} else {
							// strategy ai moves after user move
							if strategy {
								if let BoardReply::ClearDrop(_lc, _atk, _raw) =
									display.board_reply
								{
									moveflag = true;
								}
								if let BoardReply::PlainDrop(_gg) =
									display.board_reply
								{
									moveflag = true;
								}
							}
							last_display[1] = Some(display);
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
					if let Some(decoded) = last_display[0].take() {
						let mut opflag = true;
						let mut displays = vec![decoded];
						if let Some(decoded) = last_display[1].take() {
							displays.push(decoded);
						}
						for operation in self.main_think(displays).iter() {
							client_socket
								.send(ClientMsg::KeyEvent(0, *operation))
								.unwrap();
							opflag = false;
							std::thread::sleep(
								std::time::Duration::from_millis(40),
							);
						}
						moveflag = opflag;
					}
				}
			} else if let Some(decoded) = last_display[0].take() {
				if state == 2 {
					let mut displays = vec![decoded];
					if let Some(decoded) = last_display[1].take() {
						displays.push(decoded);
					}
					operation_queue = self.main_think(displays);
					while let Some(key_type) = operation_queue.pop_front() {
						client_socket
							.send(ClientMsg::KeyEvent(0, key_type))
							.unwrap();
						std::thread::sleep(std::time::Duration::from_millis(
							sleep_millis,
						));
					}
				}
				last_display = [None, None];
			}
		}
	}
}
