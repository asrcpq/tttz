extern crate termion;
use termion::async_stdin;
use termion::raw::IntoRawMode;
use std::io::{Read, stdout, Write};
extern crate bincode;
use std::net::{SocketAddr, ToSocketAddrs};
use std::net::UdpSocket;

mod client_display;
use client_display::ClientDisplay;

extern crate mpboard;
use mpboard::display::Display;

fn main() {
	let mut iter = std::env::args();
	iter.next();
	let addr = match iter.next() {
		Some(string) => string,
		None => "127.0.0.1:23124".to_string(),
	};

	let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
	let target_addr: SocketAddr = addr.to_socket_addrs()
		.unwrap()
		.next()
		.unwrap();
	eprintln!("{:?}", target_addr);
	socket.send_to(b"new client", &target_addr).unwrap();
	let mut buf = [0; 1024];
	let (amt, _) = socket.recv_from(&mut buf).unwrap();
	assert!(std::str::from_utf8(&buf).unwrap().starts_with("ok"));
	let id: i32 = std::str::from_utf8(&buf[3..amt]).unwrap().parse::<i32>().unwrap();
	socket.set_nonblocking(true);

	let stdout = stdout();
	let mut stdout = stdout.lock().into_raw_mode().unwrap();
	let mut stdin = async_stdin().bytes();
	let mut client_display = ClientDisplay::new();

	loop {
		if let Ok(amt) = socket.recv(&mut buf) {
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
					socket.send_to(b"quit", target_addr).unwrap();
					break;
				},
				b'0' => { // auto match
					socket.send_to(
						format!("get clients").as_bytes(),
						target_addr
					).unwrap();
					socket.set_nonblocking(false);
					let amt = socket.recv(&mut buf).unwrap();
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
					socket.send_to(
						format!("attack {}", max_id).as_bytes(),
						target_addr
					).unwrap();
					socket.send_to(
						format!("view {}", max_id).as_bytes(),
						target_addr
					).unwrap();
					socket.set_nonblocking(true);
				},
				_ => {
					socket
						.send_to(format!("key {}", byte as char).as_bytes(), target_addr)
						.unwrap();
				},
				_ => {},
			}
			stdout.flush();
		}
		std::thread::sleep(std::time::Duration::from_millis(10));
	}
	client_display.deinit();
}
