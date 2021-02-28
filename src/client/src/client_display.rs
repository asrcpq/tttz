extern crate termion;
use std::io::Write;

extern crate mpboard;
use mpboard::display::Display;
use mpboard::srs_data::*;

use std::collections::HashMap;

pub struct ClientDisplay {
	last_dirtypos: Vec<Vec<(u32, u32)>>,
	offset_x: Vec<u32>,
	offset_y: Vec<u32>,
}

impl ClientDisplay {
	pub fn new() -> ClientDisplay {
		// goto raw mode after ok
		print!("{}{}", termion::clear::All, termion::cursor::Hide);
		std::io::stdout().flush().unwrap();
		ClientDisplay {
			last_dirtypos: vec![vec![]; 2],
			offset_x: vec![5, 40],
			offset_y: vec![2, 2],
		}
	}

	fn blockp(&self, i: u8, j: u8, color: u8, style: u8) {
		let (ch1, ch2) = if style == 0 && color != 7 {
			('[', ']')
		} else {
			(' ', ' ')
		};
		print!(
			"[4{}m{}{}{}{}",
			COLORMAP[color as usize],
			termion::cursor::Goto(i as u16, j as u16),
			ch1,
			termion::cursor::Goto(i as u16 + 1, j as u16),
			ch2,
		);
	}

	pub fn disp_msg(&self, msg: &str, offsetx: u8, mut offsety: u8) {
		offsety += 22;
		print!(
			"{}{}{}",
			termion::style::Reset,
			termion::cursor::Goto(offsetx as u16, offsety as u16),
			msg
		);
	}

	fn disp_info(&self, display: &Display, mut offsetx: u8, mut offsety: u8) {
		offsetx += 0;
		offsety += 20;
		print!(
			"{}id: {}",
			termion::cursor::Goto(offsetx as u16, offsety as u16,),
			display.id,
		);
		print!(", combo: {}", display.combo);
		// old direct print next
		// for i in 0..n {
		// 	print!("{}{}",
		// 		termion::cursor::Goto(
		// 			(offsetx + i) as u16,
		// 			(offsety + 1) as u16,
		// 		),
		// 		ID_TO_CHAR[display.bag_preview[i as usize] as usize],
		// 	);
		// }
	}

	fn mini_blockp(&mut self, x: u32, double_y: u32, code: u8, panel: u32) {
		let mut print_info: HashMap<(u32, u32), i32> = HashMap::new();
		for i in 0..4 {
			let bpt_offset = 32 * code + i * 2;
			let x1 = BPT[bpt_offset as usize] as u32 + x;
			let double_y1 = BPT[bpt_offset as usize + 1] as u32 + double_y;
			let y1 = double_y1 / 2;
			let mut mod2 = double_y1 as i32 % 2 + 1;
			if let Some(old_mod2) = print_info.remove(&(x1, y1)) {
				mod2 |= old_mod2;
			}
			print_info.insert((x1, y1), mod2);
		}
		print!("[3{}m", COLORMAP[code as usize]);

		for ((x, y), value) in &print_info {
			// value should not be zero
			let print_char = match value {
				1 => "\u{2580}",
				2 => "\u{2584}",
				3 => "\u{2588}",
				_ => unreachable!(),
			};
			print!(
				"{}{}",
				termion::cursor::Goto(*x as u16, *y as u16),
				print_char,
			);
			self.last_dirtypos[panel as usize].push((*x, *y))
		}
		print!("{}", termion::style::Reset);
	}

	fn disp_box(&mut self, left: u16, right: u16, top: u16, bot: u16) {
		for yy in [top, bot].iter() {
			print!("{}", termion::cursor::Goto(left + 1, *yy));
			for _ in left+1..right {
				print!("\u{2500}");
			}
		}
		for xx in [left, right].iter() {
			for yy in top+1..bot {
				print!("{}\u{2502}", termion::cursor::Goto(*xx, yy));
			}
		}
		print!("{}\u{250c}{}\u{2514}{}\u{2510}{}\u{2518}",
			termion::cursor::Goto(left, top),
			termion::cursor::Goto(left, bot),
			termion::cursor::Goto(right, top),
			termion::cursor::Goto(right, bot),
		);
	}

	fn disp_hold_next(&mut self, n: usize, display: &Display, panel: u32) {
		let offsetx = 23 + self.offset_x[panel as usize] as u16;
		let offsety = self.offset_y[panel as usize] as u16 + 1;
		let mut doubley = offsety * 2;
		for (x, y) in self.last_dirtypos[panel as usize].drain(..) {
			print!("{} ", termion::cursor::Goto(x as u16, y as u16));
		}
		self.disp_box(offsetx - 1, offsetx + 4, offsety - 1, offsety + 1);
		for code in [display.hold].iter().chain(display.bag_preview[..n].iter()) {
			if *code == 7 {
				doubley += 5;
				continue
			}
			let mut tmpx = offsetx;
			if *code == 3 {
				tmpx += 1;
			}
			self.mini_blockp(
				tmpx as u32,
				doubley as u32,
				*code,
				panel,
			);
			doubley += 5;
		}
	}

	pub fn disp(&mut self, display: Display, panel: u32) {
		let offsetx = self.offset_x[panel as usize] as u8;
		let offsety = self.offset_y[panel as usize] as u8;
		for i in 0..10 {
			for j in 20..40 {
				self.blockp(
					offsetx + i * 2,
					offsety + j - 20,
					display.color[i as usize + j as usize * 10],
					0,
				);
			}
		}
		// show shadow_block first
		for i in 0..4 {
			let x = display.shadow_pos[i * 2];
			let y = display.shadow_pos[i * 2 + 1];
			if y >= 20 {
				self.blockp(offsetx + x * 2, offsety + y - 20, display.shadow_code, 1);
			}
		}
		for i in 0..4 {
			let x = display.tmp_pos[i * 2];
			let y = display.tmp_pos[i * 2 + 1];
			if y >= 20 {
				self.blockp(offsetx + x * 2, offsety + y - 20, display.tmp_code, 0);
			}
		}
		print!("{}", termion::style::Reset);
		self.disp_info(&display, offsetx, offsety);
		self.disp_hold_next(6, &display, panel);
		self.disp_atk(display.pending_attack, panel);
	}

	pub fn disp_atk(&self, atk: u32, panel: u32) {
		let offsetx = self.offset_x[panel as usize] + 20;
		let offsety = self.offset_y[panel as usize];
		print!("{}", termion::style::Reset);
		for i in 0..(20 - atk as u16) {
			print!(
				"{} ",
				termion::cursor::Goto(offsetx as u16, offsety as u16 + i),
			);
		}
		print!(
			"{}",
			if atk < 4 {
				"[43m"
			} else if atk < 10 {
				"[41m"
			} else if atk < 20 {
				"[45m"
			} else {
				"[46m"
			}
		);
		for i in (20 - atk as u16)..20 {
			print!(
				"{} ",
				termion::cursor::Goto(offsetx as u16, offsety as u16 + i),
			);
		}
		print!("{}", termion::style::Reset);
	}

	pub fn deinit(&mut self) {
		print!(
			"{}{}{}",
			termion::style::Reset,
			termion::clear::All,
			termion::cursor::Show,
		);
	}
}
