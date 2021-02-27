// stupid ai, put block to make least holes and lowest height

extern crate bincode;
extern crate rand;
use std::net::SocketAddr;
use std::net::UdpSocket;
extern crate mpboard;
use mpboard::display::Display;
use mpboard::srs_data::*;
use std::io::{self, BufRead};

fn main_think(display: Display, socket: &UdpSocket, target_addr: SocketAddr) {
	let mut heights = [0u8; 10];

	// calc height
	for i in 0..10 {
		let mut j: i32 = 0;
		while display.color[(i + j * 10) as usize] == 7 {
			j += 1;
			if j >= 40 {
				break
			}
		}
		j -= 1;
		heights[i as usize] = j as u8;
	}

	let mut best_score: f32 = 0.0;
	let mut best_rotation = 0;
	let mut best_posx = 0;
	for rot in 0..4 {
		let mut dx = 0;
		loop {
			if dx + BLOCK_WIDTH[display.tmp_code as usize * 4 + rot as usize] > 10 {
				break
			}

			let mut posx = [0; 4];
			let mut posy = [0; 4];
			for block in 0..4 {
				let offset = display.tmp_code * 32 + rot * 8 + block * 2;
				posx[block as usize] = BPT[offset as usize];
				posy[block as usize] = BPT[(offset + 1) as usize];
			}
			let mut height = 0;
			'movedown_check: loop {
				for block in 0..4 {
					if posy[block] + height ==
						(heights[dx as usize + posx[block] as usize]) as i32 {
						height -= 1;
						break 'movedown_check;
					}
				}
				height += 1;
			}

			let mut delta_heights = [0; 4];
			for block in 0..4 {
				let dh = heights[dx as usize + posx[block] as usize] as i32 - posy[block] - height;
				if dh > delta_heights[posx[block] as usize] {
					delta_heights[posx[block] as usize] = dh;
				}
			}
			let hole: i32 = delta_heights.iter().fold(0, |sum, x| sum + x) - 4; // 4 blocks
			let score = height as f32 - hole as f32 * 2.0; 
			if score > best_score {
				println!("{} overtake {} at dx: {}, rot: {}",
					score,
					best_score,
					dx,
					rot,
				);
				best_score = score;
				best_rotation = rot;
				best_posx = dx;
			}
			dx += 1;
		}
	}

	// perform action
	let mut current_posx = INITIAL_POS[display.tmp_code as usize];
	let mut rotated_pos0 = current_posx +
		SRP[(display.tmp_code * 8 + best_rotation * 2) as usize];
	let (keycode, times) = if rotated_pos0 > best_posx { //left
			('h', rotated_pos0 - best_posx)
		} else {
			('l', best_posx - rotated_pos0)
		};
	for _ in 0..best_rotation {
		socket
			.send_to(format!("key x").as_bytes(), target_addr)
			.unwrap();
	}
	for _ in 0..times {
		socket
			.send_to(format!("key {}", keycode).as_bytes(), target_addr)
			.unwrap();
	}
	socket
		.send_to(format!("key k").as_bytes(), target_addr)
		.unwrap();
}

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

	let mut amt = 0;
	loop {
		std::thread::sleep(std::time::Duration::from_millis(500));
		// let line1 = stdin.lock().lines().next().unwrap().unwrap();

		// read until last screen
		let mut buf2 = [0; 1024];
		loop {
			match socket.recv(&mut buf2) {
				Ok(amt2) => {
					if amt2 >= 16 {
						for i in 0..amt2 {
							buf[i] = buf2[i];
						}
						amt = amt2;
					} else {
						eprintln!("Short msg");
						continue
					}
				}
				Err(_) => {
					break
				}
			}
		}

		match bincode::deserialize::<Display>(&buf[..amt]) {
			Ok(decoded) => {
				if decoded.id == id {
					main_think(decoded, &socket, target_addr);
				} else {
					eprintln!("Get wrong message {}, I am {}", decoded.id, id);
				}
			},
			Err(_) => {
				eprintln!("Deserialize error");
			},
		}
	}
}
