extern crate termion;
use std::io::Write;

extern crate mpboard;
use mpboard::display::Display;
use mpboard::srs_data::*;

use std::collections::HashMap;

pub struct ClientDisplay {
	last_dirtypos: Vec<Vec<(u32, u32)>>,
	termsize: (u16, u16),
	offset_x: Vec<i32>,
	offset_y: Vec<i32>,
}

impl ClientDisplay {
	pub fn new() -> ClientDisplay {
		// goto raw mode after ok
		print!("{}{}", termion::clear::All, termion::cursor::Hide);
		std::io::stdout().flush().unwrap();
		let mut client_display = ClientDisplay {
			last_dirtypos: vec![vec![]; 2],
			termsize: (0, 0),
			offset_x: vec![-1; 2],
			offset_y: vec![-1; 2],
		};
		client_display.set_offset();
		client_display
	}

	fn checksize(&self) -> bool {
		if self.offset_x[0] == -1 {
			print!(
				"{}{}At least 24r63c size is required.",
				termion::clear::All,
				termion::cursor::Goto(1, 1),
			);
			return false;
		}
		true
	}

	pub fn set_offset(&mut self) {
		const DRAW_SIZE: (u16, u16) = (63, 24);
		let (x, y) = termion::terminal_size().unwrap();
		if (x, y) == self.termsize {
			return;
		} else {
			print!("{}", termion::clear::All);
			self.termsize = (x, y);
		}
		if x < DRAW_SIZE.0 || y < DRAW_SIZE.1 {
			self.offset_x[0] = -1;
			return;
		}
		let x1 = (x - DRAW_SIZE.0) / 2;
		let y1 = (y - DRAW_SIZE.1) / 2;
		self.disp_box(x1, x1 + DRAW_SIZE.0, y1, y1 + DRAW_SIZE.1);
		self.offset_x[0] = x1 as i32 + 4;
		self.offset_y[0] = y1 as i32 + 2;
		self.offset_x[1] = x1 as i32 + 34;
		self.offset_y[1] = y1 as i32 + 2;
	}

	fn blockp(&self, i: u16, j: u16, color: u8, style: u8) {
		let (ch1, ch2) = if style == 0 && color != 7 {
			('[', ']')
		} else {
			(' ', ' ')
		};
		print!(
			"[4{}m{}{}{}{}",
			COLORMAP[color as usize],
			termion::cursor::Goto(i, j),
			ch1,
			termion::cursor::Goto(i + 1, j),
			ch2,
		);
	}

	pub fn disp_msg(&self, msg: &str) {
		if !self.checksize() {
			return;
		}
		let offsetx = self.offset_x[0] as u16 + 1;
		let offsety = self.offset_y[0] as u16 + 21;
		print!(
			"{}{}{}{}",
			termion::cursor::Goto(offsetx, offsety),
			" ".repeat(16),
			termion::cursor::Goto(offsetx, offsety),
			msg
		);
	}

	fn disp_info(&self, display: &Display, mut offsetx: u16, mut offsety: u16) {
		const LEN: usize = 24;
		offsetx += 0;
		offsety += 20;
		let mut infostring = format!("id: {}", display.id);
		if display.combo > 0 {
			infostring = format!("{}, combo: {}", infostring, display.combo,);
		}
		if display.b2b {
			infostring += ", b2b on";
		}
		let infostring = infostring.into_bytes();
		for x in 0..LEN {
			if x < infostring.len() {
				print!(
					"{}{}",
					termion::cursor::Goto(offsetx + x as u16, offsety,),
					infostring[x] as char,
				);
			} else {
				print!("{} ", termion::cursor::Goto(offsetx + x as u16, offsety,),);
			}
		}
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
			for _ in left + 1..right {
				print!("\u{2500}");
			}
		}
		for xx in [left, right].iter() {
			for yy in top + 1..bot {
				print!("{}\u{2502}", termion::cursor::Goto(*xx, yy));
			}
		}
		print!(
			"{}\u{250c}{}\u{2514}{}\u{2510}{}\u{2518}",
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
				continue;
			}
			let mut tmpx = offsetx;
			if *code == 3 {
				tmpx += 1;
			}
			self.mini_blockp(tmpx as u32, doubley as u32, *code, panel);
			doubley += 5;
		}
	}

	pub fn disp(&mut self, display: Display, panel: u32) {
		if !self.checksize() {
			return;
		}
		print!("[30m");
		let offsetx = self.offset_x[panel as usize] as u16;
		let offsety = self.offset_y[panel as usize] as u16;
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
			let x = display.shadow_pos[i * 2] as u16;
			let y = display.shadow_pos[i * 2 + 1] as u16;
			if y >= 20 {
				self.blockp(offsetx + x * 2, offsety + y - 20, display.shadow_code, 1);
			}
		}
		for i in 0..4 {
			let x = display.tmp_pos[i * 2] as u16;
			let y = display.tmp_pos[i * 2 + 1] as u16;
			if y >= 20 {
				self.blockp(offsetx + x * 2, offsety + y - 20, display.tmp_code, 0);
			}
		}
		print!("{}", termion::style::Reset);
		self.disp_info(&display, offsetx, offsety);
		self.disp_hold_next(6, &display, panel);
		self.disp_atk(display.pending_attack, panel);
	}

	pub fn disp_atk_pub(&self, atk: u32, panel: u32) {
		if !self.checksize() {
			return;
		}
		self.disp_atk(atk, panel);
	}

	fn disp_atk(&self, atk: u32, panel: u32) {
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
