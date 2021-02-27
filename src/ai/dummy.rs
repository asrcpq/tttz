// dummy ai, only hard drop blocks

extern crate bincode;
extern crate rand;
use std::net::SocketAddr;
use std::net::UdpSocket;
extern crate mpboard;
use mpboard::display::Display;
use mpboard::srs_data::*;
use std::io::{self, BufRead};

fn main() {
	let stdin = io::stdin();

	let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
	let target_addr: SocketAddr = "127.0.0.1:23124".parse().unwrap();
	socket.send_to(b"new client", &target_addr).unwrap();
	let mut buf = [0; 1024];
	let (amt, _) = socket.recv_from(&mut buf).unwrap();
	assert!(std::str::from_utf8(&buf).unwrap().starts_with("ok"));
	let id: i32 = std::str::from_utf8(&buf[3..amt]).unwrap().parse::<i32>().unwrap();

	// auto match
	socket.send_to(
		format!("get clients").as_bytes(),
		target_addr
	).unwrap();
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

	// start after read something
	let line1 = stdin.lock().lines().next().unwrap().unwrap();
	socket
		.send_to(format!("key r").as_bytes(), target_addr)
		.unwrap();
	socket.set_nonblocking(true);

	loop {
		socket
			.send_to(format!("key k").as_bytes(), target_addr)
			.unwrap();
		std::thread::sleep(std::time::Duration::from_millis(1000));
	}
}
