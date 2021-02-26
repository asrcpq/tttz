extern crate termion;
use termion::event::{Event, Key};
use termion::input::{MouseTerminal, TermRead};
use termion::raw::IntoRawMode;
extern crate lazy_static;
extern crate rand;
use std::io::{stdin, stdout, Write};
use std::net::SocketAddr;
use std::net::UdpSocket;

fn main() {
	let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
	let target_addr: SocketAddr = "127.0.0.1:23124".parse().unwrap();
	socket.send_to(b"new cc", &target_addr).unwrap();
	let mut buf = [0; 1024];
	let (amt, _) = socket.recv_from(&mut buf).unwrap();
	let id = std::str::from_utf8(&buf[..amt])
		.unwrap()
		.parse::<i32>()
		.unwrap();
	println!("My id: {}", id);

	// open socket first, or the id wait cannot be sigint
	let mut stdout = MouseTerminal::from(stdout().into_raw_mode().unwrap());
	write!(stdout, "{}{}", termion::clear::All, termion::cursor::Hide).unwrap();
	let stdin = stdin();
	for c in stdin.events() {
		let evt = c.unwrap();
		match evt {
			Event::Key(Key::Char('q')) => {
				socket.send_to(b"quit", target_addr).unwrap();
				break;
			}
			Event::Key(Key::Char(ch)) => {
				socket
					.send_to(format!("key {}", ch).as_bytes(), target_addr)
					.unwrap();
			}
			_ => {}
		}
	}
	print!("[0;0m{}{}", termion::clear::All, termion::cursor::Show);
}
