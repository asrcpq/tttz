extern crate termion;
use std::io::{stdout, Read, Write};
use termion::async_stdin;
use termion::raw::IntoRawMode;
extern crate bincode;

mod client_display;
use client_display::ClientDisplay;
mod client_socket;
use client_socket::ClientSocket;

extern crate mpboard;
use mpboard::display::Display;

fn main() {
	let mut text_mode = false; // starting with /, client type a message to server
	let mut text: Vec<u8> = Vec::new();

	let mut iter = std::env::args();
	iter.next();
	let addr = match iter.next() {
		Some(string) => string,
		None => "127.0.0.1:23124".to_string(),
	};

	let (client_socket, id) = ClientSocket::new(&addr);

	let stdout = stdout();
	let mut stdout = stdout.lock().into_raw_mode().unwrap();
	let mut stdin = async_stdin().bytes();
	let mut client_display = ClientDisplay::new();

	let mut state = 1;
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
				} else if msg == "start" {
					state = 2;
				} else if msg == "die" || msg == "win" {
					state = 1;
				}
				client_display.disp_msg(&msg);
				continue;
			} else {
				let decoded: Display =
					bincode::deserialize(&buf[..amt]).unwrap();
				if decoded.id == id {
					client_display.disp(decoded, 0);
				} else {
					client_display.disp(decoded, 1);
				}
			}
			stdout.flush().unwrap();
		}
		if let Some(Ok(byte)) = stdin.next() {
			if text_mode {
				if byte == b'/' {
					client_socket.send(&text).unwrap();
					text = Vec::new();
					text_mode = false;
				} else {
					text.push(byte);
				}
			} else {
				match byte {
					b'q' => {
						break;
					}
					b'r' => {
						if state == 2 {
							client_socket.send(b"suicide").unwrap();
							state = 3;
						} else {
							client_socket.send(b"pair").unwrap();
							state = 3;
						}
					}
					b'/' => {
						text_mode = true;
					}
					_ => {
						if state == 2 {
							client_socket
								.send(
									format!("key {}", byte as char).as_bytes(),
								)
								.unwrap();
						}
					}
				}
			}
		}
		std::thread::sleep(std::time::Duration::from_millis(10));
	}
	client_display.deinit();
}
