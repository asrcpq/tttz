extern crate termion;
use termion::async_stdin;
use termion::raw::IntoRawMode;
use std::io::{Read, stdout, Write};
extern crate bincode;

mod client_display;
use client_display::ClientDisplay;
mod client_socket;
use client_socket::ClientSocket;

extern crate mpboard;
use mpboard::display::Display;

fn main() {
	let mut iter = std::env::args();
	iter.next();
	let addr = match iter.next() {
		Some(string) => string,
		None => "127.0.0.1:23124".to_string(),
	};

	let (mut client_socket, id) = ClientSocket::new(&addr);

	let stdout = stdout();
	let mut stdout = stdout.lock().into_raw_mode().unwrap();
	let mut stdin = async_stdin().bytes();
	let mut client_display = ClientDisplay::new();

	let mut buf = [0; 1024];
	loop {
		if let Ok(amt) = client_socket.recv(&mut buf) {
			// all long messages are board display
			if amt < 16 {
				let msg = std::str::from_utf8(&buf[..amt]).unwrap();
				if msg.starts_with("sigatk ") {
					let pending_atk = msg[7..amt].parse::<u32>().unwrap();
					client_display.disp_atk(pending_atk, 1, 1);
				}
				continue
			} else {
				let decoded: Display = bincode::deserialize(&buf[..amt]).unwrap();
				if decoded.id == id {
					client_display.disp(decoded, 1, 1);
				} else {
					client_display.disp(decoded, 31, 1);
				}
			}
			stdout.flush();
		}
		if let Some(Ok(byte)) = stdin.next() {
			match byte {
				b'q' => {
					client_socket.send(b"quit").unwrap();
					break;
				},
				b'0' => { // auto match
					client_socket.send(format!("get clients").as_bytes()).unwrap();
					client_socket.set_nonblocking(false);
					let amt = client_socket.recv(&mut buf).unwrap();
					// find latest client
					let mut max_id = 0;
					for each_str in String::from(std::str::from_utf8(&buf[..amt]).unwrap())
						.split_whitespace().rev() {
						if let Ok(each_id) = each_str.parse::<i32>() {
							if id != each_id && id > max_id {
								max_id = each_id;
							}
						}
					}
					client_socket.send(format!("attack {}", max_id).as_bytes()).unwrap();
					client_socket.send(format!("view {}", max_id).as_bytes()).unwrap();
					client_socket.set_nonblocking(true);
				},
				_ => {
					client_socket.send(format!("key {}", byte as char).as_bytes()).unwrap();
				},
				_ => {},
			}
			stdout.flush();
		}
		std::thread::sleep(std::time::Duration::from_millis(10));
	}
	client_display.deinit();
}
