extern crate bincode;
extern crate termion;
use termion::input::MouseTerminal;
use termion::async_stdin;
use termion::raw::IntoRawMode;
extern crate lazy_static;
extern crate rand;
use std::io::{Read, stdout, Write};
use std::net::SocketAddr;
use std::net::UdpSocket;
extern crate mpboard;
use mpboard::display::Display;
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

fn disp_next(n: u8, id: i32, hold: u8, data: &[u8], mut offsetx: u8, mut offsety: u8) {
	offsetx += 1;
	offsety += 21;
	println!("{}id: {}, hold: {}",
		termion::cursor::Goto(
			offsetx as u16,
			offsety as u16,
		),
		id,
		ID_TO_CHAR[hold as usize],
	);
	for i in 0..n {
		println!("{}{}",
			termion::cursor::Goto(
				(offsetx + i) as u16,
				(offsety + 1) as u16,
			),
			ID_TO_CHAR[data[i as usize] as usize],
		);
	}
}

fn disp(display: Display, offsetx: u8, offsety: u8) {
	for i in 0..10 {
		for j in 20..40 {
			blockp(offsetx + i, offsety + j, display.color[i as usize + j as usize * 10], 0);
		}
	}
	// show shadow_block first
	for i in 0..4 {
		let x = display.shadow_pos[i * 2];
		let y = display.shadow_pos[i * 2 + 1];
		blockp(offsetx + x, offsety + y, display.shadow_code, 1);
	}
	for i in 0..4 {
		let x = display.tmp_pos[i * 2];
		let y = display.tmp_pos[i * 2 + 1];
		blockp(offsetx + x, offsety + y, display.tmp_code, 0);
	}
	println!("{}", termion::style::Reset);
	disp_next(6, display.id, display.hold, &display.bag_preview, offsetx, offsety);
}

fn main() {
	let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
	let target_addr: SocketAddr = "127.0.0.1:23124".parse().unwrap();
	socket.send_to(b"new client", &target_addr).unwrap();
	let mut buf = [0; 1024];
	let (amt, _) = socket.recv_from(&mut buf).unwrap();
	assert!(std::str::from_utf8(&buf).unwrap().starts_with("ok"));
	let id: i32 = std::str::from_utf8(&buf[3..amt]).unwrap().parse::<i32>().unwrap();
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
			let decoded: Display = bincode::deserialize(&buf[..amt]).unwrap();
			if decoded.id == id {
				disp(decoded, 0, 0);
			} else {
				disp(decoded, 15, 0);
			}
			stdout.flush().unwrap();
		}
		if let Some(Ok(byte)) = stdin.next() {
			match byte {
				b'q' => {
					socket.send_to(b"quit", target_addr).unwrap();
					break;
				},
				b'0'..=b'9' => {
					socket.send_to(format!("view {}", byte as char).as_bytes(), target_addr).unwrap();
				},
				_ => {
					socket
						.send_to(format!("key {}", byte as char).as_bytes(), target_addr)
						.unwrap();
				},
				_ => {},
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
