// stupid ai, put block to make least holes and lowest height

extern crate tttz_mpboard;
use tttz_mpboard::srs_data::*;
extern crate tttz_protocol;
use tttz_protocol::Display;
use tttz_protocol::{ClientMsg, KeyType, ServerMsg};

extern crate tttz_libclient;
use tttz_libclient::client_socket::ClientSocket;

use std::collections::VecDeque;

fn main_think(display: &Display) -> VecDeque<KeyType> {
	let mut heights = [39u8; 10];

	let mut ret = VecDeque::new();

	if display.hold == 7 {
		ret.push_back(KeyType::Hold);
		return ret;
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
					break;
				}
			} else if state == 0 {
				state = 1;
				heights[i as usize] = j as u8 - 1;
			}
			j += 1;
			if j == 40 {
				break;
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
	for (id, option_code) in [display.tmp_block[2], display.hold].iter().enumerate()
	{
		for rot in 0..4 {
			let mut dx = 0;
			loop {
				if dx + BLOCK_WIDTH[*option_code as usize * 4 + rot as usize]
					> 10
				{
					break;
				}

				let mut posx = [0; 4];
				let mut posy = [0; 4];
				for block in 0..4 {
					let offset = option_code * 32 + rot * 8 + block * 2;
					posx[block as usize] = BPT[offset as usize];
					posy[block as usize] = BPT[(offset + 1) as usize];
				}
				let mut posy_sum = 0;
				for each_posy in posy.iter() {
					posy_sum += each_posy;
				}
				let mut height = 0;
				'movedown_check: loop {
					for block in 0..4 {
						if posy[block] + height
							== (heights[dx as usize + posx[block] as usize])
								as i32
						{
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
						- posy[block] - height;
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
				let cover = (dx <= highest_hole_x
					&& dx
						+ BLOCK_WIDTH[*option_code as usize * 4 + rot as usize]
						> highest_hole_x) as i32;
				let score = height as f32 + posy_sum as f32 * 0.25 // mass center height
					- hole as f32 - cover as f32 * 2.0;
				if score > best_score {
					eprintln!(
						"{} {} {} = {} overtake {} at dx: {}, rot: {}",
						height, hole, cover, score, best_score, dx, rot,
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
		display.tmp_block[2]
	} else {
		// best solution is from the hold block
		ret.push_back(KeyType::Hold);
		display.hold
	};
	// perform action
	let current_posx = INITIAL_POS[best_code as usize];
	let rotated_pos0 =
		current_posx + SRP[(best_code * 8 + best_rotation * 2) as usize];
	let (keycode, times) = if best_posx == 0 {
		(KeyType::LLeft, 1)
	} else if best_posx
		== 10 - BLOCK_WIDTH[(best_code * 4 + best_rotation) as usize]
	{
		(KeyType::RRight, 1)
	} else if rotated_pos0 > best_posx {
		(KeyType::Left, rotated_pos0 - best_posx)
	} else {
		(KeyType::Right, best_posx - rotated_pos0)
	};
	if best_rotation == 1 {
		ret.push_back(KeyType::Rotate);
	} else if best_rotation == 3 {
		ret.push_back(KeyType::RotateReverse);
	} else if best_rotation == 2 {
		ret.push_back(KeyType::RotateFlip);
	}
	for _ in 0..times {
		ret.push_back(keycode.clone());
	}
	ret.push_back(KeyType::HardDrop);
	ret
}

pub fn main(addr: &str, sleep_millis: u64, strategy: bool) {
	let (client_socket, id) = ClientSocket::new(&addr);
	let main_sleep = 10;

	let mut state = 3;
	let mut last_display: Option<Display> = None;
	let mut moveflag = false;
	let mut operation_queue: VecDeque<KeyType> = VecDeque::new();
	loop {
		std::thread::sleep(std::time::Duration::from_millis(main_sleep));
		// read until last screen
		while let Ok(server_msg) = client_socket.recv() {
			match server_msg {
				ServerMsg::Display(display) => {
					if display.id == id {
						last_display = Some(display.into_owned());
					} else {
						// strategy ai moves after user move
						if strategy {
							moveflag = true;
						}
					}
				}
				ServerMsg::GameOver(_) => {
					state = 1;
				}
				ServerMsg::Start(_) => {
					state = 2;
				}
				ServerMsg::Request(id) => {
					state = 2;
					client_socket.send(ClientMsg::Accept(id)).unwrap();
				}
				ServerMsg::Terminate => {
					return;
				}
				_ => eprintln!("Skipping msg: {}", server_msg),
			}
		}
		if strategy {
			if let Some(ref decoded) = last_display {
				if state == 2 && moveflag {
					if operation_queue.is_empty() {
						operation_queue = main_think(decoded);
					}
					client_socket
						.send(ClientMsg::KeyEvent(
							operation_queue.pop_front().unwrap(),
						))
						.unwrap();
					moveflag = false;
				}
				last_display = None;
			}
		} else if let Some(ref decoded) = last_display {
			if state == 2 {
				operation_queue = main_think(decoded);
				while let Some(key_type) = operation_queue.pop_front() {
					client_socket.send(ClientMsg::KeyEvent(key_type)).unwrap();
					std::thread::sleep(std::time::Duration::from_millis(
						sleep_millis,
					));
				}
			}
			last_display = None;
		}
	}
}
