// stupid ai, put block to make least holes and lowest height

extern crate bincode;
use std::net::SocketAddr;
use std::net::UdpSocket;
extern crate mpboard;
use mpboard::display::Display;
use mpboard::srs_data::*;
use std::io::{self, BufRead};

const SLEEP_MILLIS: u64 = 200;

fn main_think(display: Display, socket: &UdpSocket, target_addr: SocketAddr) {
	let mut heights = [39u8; 10];

	if display.hold == 7 {
		socket
			.send_to((b"key  "), target_addr)
			.unwrap();
		return;
	}

	// calc height
	let mut highest_hole = 40;
	let mut highest_hole_x = -1;
	for i in 0..10 {
		let mut j: i32 = 0;
		let mut state = 0;
		loop {
			if display.color[(i + j * 10) as usize] == 7 {
				if state == 1 {
					break
				}
			} else {
				if state == 0 {
					state = 1;
					heights[i as usize] = j as u8 - 1;
				}
			}
			j += 1;
			if j == 40 {
				break
			}
		}
		if j > highest_hole {
			highest_hole = j;
			highest_hole_x = i;
		}
	}

	let mut best_score: f32 = 0.0;
	let mut best_rotation = 0;
	let mut best_posx = 0;
	let mut best_id = 0;
	for (id, option_code) in [display.tmp_code, display.hold].iter().enumerate() {
		for rot in 0..4 {
			let mut dx = 0;
			loop {
				if dx + BLOCK_WIDTH[*option_code as usize * 4 + rot as usize] > 10 {
					break
				}

				let mut posx = [0; 4];
				let mut posy = [0; 4];
				for block in 0..4 {
					let offset = option_code * 32 + rot * 8 + block * 2;
					posx[block as usize] = BPT[offset as usize];
					posy[block as usize] = BPT[(offset + 1) as usize];
				}
				let mut posy_sum = 0;
				for i in 0..4 {
					posy_sum += posy[i];
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
				let mut block_count = [0; 4];
				for block in 0..4 {
					let dh = heights[dx as usize + posx[block] as usize] as i32
						- posy[block]
						- height;
					block_count[posx[block] as usize] += 1;
					if dh > delta_heights[posx[block] as usize] {
						delta_heights[posx[block] as usize] = dh;
					}
				}
				let mut hole: i32 = 0;
				for block in 0..4 {
					if delta_heights[block] > block_count[block] {
						hole += 1;
					}
				}
				let cover = (dx <= highest_hole_x &&
					dx + BLOCK_WIDTH[*option_code as usize * 4 + rot as usize] > highest_hole_x)
					as i32;
				let score = height as f32 + posy_sum as f32 * 0.25 // mass center height
					- hole as f32 - cover as f32 * 2.0;
				if score > best_score {
					println!("{} {} {} = {} overtake {} at dx: {}, rot: {}",
						height,
						hole,
						cover,
						score,
						best_score,
						dx,
						rot,
					);
					best_score = score;
					best_rotation = rot;
					best_posx = dx;
					best_id = id;
				}
				dx += 1;
			}
		}
	}

	let best_code = if best_id == 0 {
		display.tmp_code
	} else {
		// best solution is from the hold block
		socket
			.send_to((b"key  "), target_addr)
			.unwrap();
		display.hold
	};
	// perform action
	let mut current_posx = INITIAL_POS[best_code as usize];
	let mut rotated_pos0 = current_posx +
		SRP[(best_code * 8 + best_rotation * 2) as usize];
	let (keycode, times) = if rotated_pos0 > best_posx { //left
			('h', rotated_pos0 - best_posx)
		} else {
			('l', best_posx - rotated_pos0)
		};
	for _ in 0..best_rotation {
		socket
			.send_to((b"key x"), target_addr)
			.unwrap();
		std::thread::sleep(std::time::Duration::from_millis(SLEEP_MILLIS));
	}
	for _ in 0..times {
		socket
			.send_to(format!("key {}", keycode).as_bytes(), target_addr)
			.unwrap();
		std::thread::sleep(std::time::Duration::from_millis(SLEEP_MILLIS));
	}
	socket
		.send_to(b"key k", target_addr)
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

	// decide mode after client connection
	let args: Vec<String> = std::env::args().collect();
	let mode = if args.len() == 1 {
		"pair".to_string()
	} else {
		args[1].clone()
	};
	// free mode will immediately start, so pause first
	if mode == "free" {
		stdin.lock().lines().next().unwrap().unwrap();
	}

	socket
		.send_to(mode.as_bytes(), target_addr)
		.unwrap();
	socket.set_nonblocking(true);

	let mut state = 3;
	let mut display: Option<Display> = None;
	loop {
		std::thread::sleep(std::time::Duration::from_millis(SLEEP_MILLIS));

		// read until last screen
		loop {
			match socket.recv(&mut buf) {
				Ok(amt) => {
					if amt >= 16 {
						match bincode::deserialize::<Display>(&buf[..amt]) {
							Ok(decoded) => {
								if decoded.id == id {
									display = Some(decoded);
								} else {
									eprintln!("Get wrong message {}, I am {}", decoded.id, id);
								}
							},
							Err(_) => {
								eprintln!("Deserialize error");
							},
						}
					} else {
						let msg = std::str::from_utf8(&buf[..amt]).unwrap();
						if msg == "die" || msg == "win" {
							socket
								.send_to(b"pair", target_addr)
								.unwrap();
							state = 3;
						}
						if msg == "start" {
							state = 2;
						}
						eprintln!("Short msg: {}", msg);
					}
				}
				Err(_) => {
					break
				}
			}
		}
		if let Some(decoded) = display {
			if state == 2 {
				main_think(decoded, &socket, target_addr);
			}
			display = None;
		}
	}
}
