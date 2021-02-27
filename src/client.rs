extern crate termion;
use termion::event::{Event, Key};
use termion::input::{MouseTerminal, TermRead};
use termion::async_stdin;
use termion::raw::IntoRawMode;
extern crate lazy_static;
extern crate rand;
use std::io::{Read, stdout, Write};
use std::net::SocketAddr;
use std::net::UdpSocket;
extern crate mpboard;
use mpboard::block;
use mpboard::srs_data::*;

fn blockp(i: u8, mut j: u8, color: u8, style: u8) {
	if j < 20 {
		return;
	}
	j -= 20;
	let (ch1, ch2) = if style == 0 && color != 7 { ('[', ']') } else { (' ', ' ') };
	print!(
		"[4{}m{}{}{}{}",
		COLORMAP[color as usize],
		termion::cursor::Goto(1 + i as u16 * 2, 1 + j as u16),
		ch1,
		termion::cursor::Goto(1 + i as u16 * 2 + 1, 1 + j as u16),
		ch2,
	);
}

fn disp_next(n: u8, data: &[u8]) {
	let offsetx = 1;
	let offsety = 21;
	println!("{}hold: {}",
		termion::cursor::Goto(
			offsetx,
			offsety,
		),
		ID_TO_CHAR[data[0] as usize],
	);
	for i in 1..=n {
		println!("{}{}",
			termion::cursor::Goto(
				offsetx + i as u16,
				offsety + 1,
			),
			ID_TO_CHAR[data[i as usize] as usize],
		);
	}
}

fn disp(buf: &[u8]) {
	for i in 0..10 {
		for j in 0..20 {
			blockp(i, j + 20, buf[i as usize + j as usize * 10], 0);
		}
	}
	let offset = 200;
	// show shadow_block first
	for i in 0..4 {
		let x = buf[offset + i * 2];
		let y = buf[offset + i * 2 + 1];
		blockp(x, y, buf[offset + 8], 1);
	}
	let offset = 209;
	for i in 0..4 {
		let x = buf[offset + i * 2];
		let y = buf[offset + i * 2 + 1];
		blockp(x, y, buf[offset + 8], 0);
	}
	println!("{}", termion::style::Reset);
	disp_next(6, &buf[218..225]);
}

fn main() {
	let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
	let target_addr: SocketAddr = "127.0.0.1:23124".parse().unwrap();
	socket.send_to(b"new client", &target_addr).unwrap();
	let mut buf = [0; 1024];
	let (_, _) = socket.recv_from(&mut buf).unwrap();
	assert!(std::str::from_utf8(&buf).unwrap().starts_with("ok"));
	socket.set_nonblocking(true);

	// goto raw mode after ok
	let mut stdout = MouseTerminal::from(stdout().into_raw_mode().unwrap());
	write!(stdout, "{}{}", termion::clear::All, termion::cursor::Hide).unwrap();
	stdout.flush().unwrap();
	let mut stdin = async_stdin().bytes();

	loop {
		if let Ok(amt) = socket.recv(&mut buf) {
			if amt < 16 && buf[0] == b'q' {
				break;
			}
			disp(&buf[..amt]);
			stdout.flush().unwrap();
		}
		if let Some(Ok(byte)) = stdin.next() {
			match byte {
				b'q' => {
					socket.send_to(b"quit", target_addr).unwrap();
					break;
				}
				byte => {
					socket
						.send_to(format!("key {}", byte as char).as_bytes(), target_addr)
						.unwrap();
				}
				_ => {}
			}
		}
		std::thread::sleep(std::time::Duration::from_millis(10));
	}
	println!(
		"{}{}{}",
		termion::style::Reset,
		termion::clear::All,
		termion::cursor::Show,
	);
}
