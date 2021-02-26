extern crate termion;

extern crate mpboard;
use mpboard::block;
use mpboard::srs_data;

use srs_data::*;
use std::io::{stdout, Write};
use std::net::SocketAddr;
use std::net::UdpSocket;
use termion::input::MouseTerminal;
use termion::raw::IntoRawMode;

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

// fn disp_next(n: u8) {
// 	let offsetx = 1;
// 	let offsety = 21;
// 	println!("{}hold: {}",
// 		termion::cursor::Goto(
// 			offsetx,
// 			offsety,
// 		),
// 		ID_TO_CHAR[self.hold as usize],
// 	);
// 	for i in 0..n {
// 		println!("{}{}",
// 			termion::cursor::Goto(
// 				offsetx + i,
// 				offsety + 1,
// 			),
// 			ID_TO_CHAR[self.rg.bag[i as usize] as usize],
// 		);
// 	}
// }

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
	// self.disp_next(6);
}

fn main() {
	let id = std::env::args()
			.collect::<Vec<String>>()[1]
			.parse::<i32>()
			.unwrap();
	let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
	let target_addr: SocketAddr = "127.0.0.1:23124".parse().unwrap();
	socket.send_to(format!("new dc {}", id).as_bytes(), &target_addr).unwrap();
	let mut buf = [0; 1024];
	let (_, _) = socket.recv_from(&mut buf).unwrap();
	assert!(std::str::from_utf8(&buf).unwrap().starts_with("ok"));

	// goto raw mode after ok
	let mut stdout = MouseTerminal::from(stdout().into_raw_mode().unwrap());
	write!(stdout, "{}{}", termion::clear::All, termion::cursor::Hide).unwrap();
	stdout.flush().unwrap();

	loop {
		let (amt, _) = socket.recv_from(&mut buf).unwrap();
		if amt < 16 && buf[0] == b'q' {
			break;
		}
		disp(&buf[..amt]);
		stdout.flush().unwrap();
	}
	println!(
		"{}{}{}",
		termion::style::Reset,
		termion::clear::All,
		termion::cursor::Show,
	);
}
