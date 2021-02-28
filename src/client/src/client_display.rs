extern crate termion;
use termion::raw::IntoRawMode;
use std::io::{Read, Write, StdoutLock};

extern crate mpboard;
use mpboard::display::Display;
use mpboard::srs_data::*;

use std::collections::HashMap;

pub struct ClientDisplay {
	last_dirtypos: Vec<Vec<(u32, u32)>>,
}

impl ClientDisplay {
	pub fn new() -> ClientDisplay {
		// goto raw mode after ok
		print!("{}{}", termion::clear::All, termion::cursor::Hide);
		std::io::stdout().flush().unwrap();
		ClientDisplay {
			last_dirtypos: vec![vec![]; 2],
		}
	}
	
	fn blockp(&self, i: u8, mut j: u8, color: u8, style: u8) {
		if j < 20 {
			return;
		}
		j -= 20;
		let (ch1, ch2) = if style == 0 && color != 7 { ('[', ']') } else { (' ', ' ') };
		print!(
			"[4{}m{}{}{}{}",
			COLORMAP[color as usize],
			termion::cursor::Goto(i as u16, j as u16),
			ch1,
			termion::cursor::Goto(i as u16 + 1, j as u16),
			ch2,
		);
	}

	pub fn disp_msg(&self, msg: &str, mut offsetx: u8, mut offsety: u8) {
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
		print!("{}id: {}, hold: {}",
			termion::cursor::Goto(
				offsetx as u16,
				offsety as u16,
			),
			display.id,
			ID_TO_CHAR[display.hold as usize],
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
			print!("{}{}",
				termion::cursor::Goto(*x as u16, *y as u16),
				print_char,
			);
			self.last_dirtypos[panel as usize].push((*x, *y))
		}
		print!("{}", termion::style::Reset);
	}

	fn disp_hold_next(&mut self, n: usize, display: &Display, panel: u32) {
		let offsetx = if panel == 1 {
			23 + 32
		} else {
			23 + 2
		};
		let offsety = 3;
		let mut doubley = offsety * 2; 
		for (x, y) in self.last_dirtypos[panel as usize].drain(..) {
			print!("{} ", termion::cursor::Goto(x as u16, y as u16));
		}
		if display.hold != 7 {
			self.mini_blockp(offsetx as u32, doubley as u32, display.hold, panel);
		}
		for i in 0..n {
			doubley += 4;
			self.mini_blockp(offsetx as u32, doubley as u32, display.bag_preview[i], panel);
		}
	}
	
	pub fn disp(&mut self, display: Display, panel: u32) {
		let offsetx = if panel == 1 {
			32
		} else {
			2
		};
		let offsety = 2;
		for i in 0..10 {
			for j in 20..40 {
				self.blockp(offsetx + i * 2, offsety + j, display.color[i as usize + j as usize * 10], 0);
			}
		}
		// show shadow_block first
		for i in 0..4 {
			let x = display.shadow_pos[i * 2];
			let y = display.shadow_pos[i * 2 + 1];
			self.blockp(offsetx + x * 2, offsety + y, display.shadow_code, 1);
		}
		for i in 0..4 {
			let x = display.tmp_pos[i * 2];
			let y = display.tmp_pos[i * 2 + 1];
			self.blockp(offsetx + x * 2, offsety + y, display.tmp_code, 0);
		}
		print!("{}", termion::style::Reset);
		self.disp_info(&display, offsetx, offsety);
		self.disp_hold_next(6, &display, panel);
		self.disp_atk(display.pending_attack, offsetx, offsety);
	}
	
	pub fn disp_atk(&self, atk: u32, mut offsetx: u8, offsety: u8) {
		offsetx += 20;
		print!("{}", termion::style::Reset);
		for i in 0..(20 - atk as u16) {
			print!("{} ",
				termion::cursor::Goto(offsetx as u16, offsety as u16 + i),
			);
		}
		print!("{}", if atk < 4 {
			"[43m"
		} else if atk < 10 {
			"[41m"
		} else if atk < 20 {
			"[45m"
		} else {
			"[46m"
		});
		for i in (20 - atk as u16)..20 {
			print!("{} ",
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
