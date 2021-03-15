use tttz_protocol::{Display, IdType};
use tttz_ruleset::*;

use std::collections::HashMap;
use std::io::Write;

const DRAW_SIZE: (u16, u16) = (63, 24);

pub struct ClientDisplay {
	last_dirtypos: Vec<Vec<(u32, u32)>>,
	termsize: (u16, u16),
	offset0: (u16, u16),
	offset_x: Vec<i32>,
	offset_y: Vec<i32>,
	id_panel: HashMap<IdType, usize>,
}

impl Default for ClientDisplay {
	fn default() -> ClientDisplay {
		let mut client_display = ClientDisplay {
			last_dirtypos: vec![vec![]; 2],
			termsize: (0, 0),
			offset0: (1, 1),
			offset_x: vec![-1; 2],
			offset_y: vec![-1; 2],
			id_panel: HashMap::new(),
		};
		client_display.set_offset();
		client_display.deactivate(); // start from text mode
		std::io::stdout().flush().unwrap();
		client_display
	}
}

impl ClientDisplay {
	pub fn setpanel(&mut self, panel: usize, id: IdType) {
		self.id_panel.insert(id, panel);
	}

	fn checksize(&self) -> bool {
		if self.termsize.0 < 63 || self.termsize.1 < 24 {
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
		self.offset0.0 = (x - DRAW_SIZE.0) / 2;
		self.offset0.1 = (y - DRAW_SIZE.1) / 2;
		self.offset_x[0] = self.offset0.0 as i32 + 4;
		self.offset_y[0] = self.offset0.1 as i32 + 2;
		self.offset_x[1] = self.offset0.0 as i32 + 34;
		self.offset_y[1] = self.offset0.1 as i32 + 2;
		self.disp_mainbox();
	}

	pub fn disp_mainbox(&self) {
		self.disp_box(
			self.offset0.0,
			self.offset0.0 + DRAW_SIZE.0,
			self.offset0.1,
			self.offset0.1 + DRAW_SIZE.1,
		);
	}

	fn blockp(&self, i: u16, j: u16, piece: u8, style: u8) {
		let (ch1, ch2) = if piece != b' ' {
			('[', ']')
		} else {
			(' ', ' ')
		};
		let fgbg = 4 - style; // 0 bg 1 fg
		print!(
			"[{}8;5;{}m{}{}{}{}",
			fgbg,
			COLORMAP[piece],
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
		print!("{}{:<32}", termion::cursor::Goto(offsetx, offsety), msg);
	}

	fn disp_info(&self, display: &Display, mut offsetx: u16, mut offsety: u16) {
		const LEN: usize = 24;
		offsetx += 0;
		offsety += 20;
		let mut infostring = format!("id: {}", display.id);
		if display.cm > 0 {
			infostring = format!("{}, c: {}", infostring, display.cm);
		}
		if display.tcm > 0 {
			infostring = format!("{}, b: {}", infostring, display.tcm);
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
				print!(
					"{} ",
					termion::cursor::Goto(offsetx + x as u16, offsety,),
				);
			}
		}
	}

	fn mini_blockp(&mut self, x: u32, double_y: u32, code: CodeType, panel: usize) {
		let mut print_info: HashMap<(u32, u32), i32> = HashMap::new();
		for i in 0..4 {
			let tmp = BPT[code as usize][0][i as usize];
			let x1 = tmp.0 as u32 + x;
			let double_y1 = double_y + 1 - tmp.1 as u32;
			let y1 = double_y1 / 2;
			let mut mod2 = double_y1 as i32 % 2 + 1;
			if let Some(old_mod2) = print_info.remove(&(x1, y1)) {
				mod2 |= old_mod2;
			}
			print_info.insert((x1, y1), mod2);
		}
		print!("[38;5;{}m", COLORMAP[ID_TO_CHAR[code as usize]]);

		for ((x, y), value) in print_info.into_iter() {
			// value should not be zero
			let print_char = match value {
				1 => "\u{2580}",
				2 => "\u{2584}",
				3 => "\u{2588}",
				_ => unreachable!(),
			};
			print!(
				"{}{}",
				termion::cursor::Goto(x as u16, y as u16),
				print_char,
			);
			self.last_dirtypos[panel].push((x, y))
		}
		print!("{}", termion::style::Reset);
	}

	fn disp_box(&self, left: u16, right: u16, top: u16, bot: u16) {
		for &yy in [top, bot].iter() {
			print!("{}", termion::cursor::Goto(left + 1, yy));
			for _ in left + 1..right {
				print!("\u{2500}");
			}
		}
		for &xx in [left, right].iter() {
			for yy in top + 1..bot {
				print!("{}\u{2502}", termion::cursor::Goto(xx, yy));
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

	fn disp_hold_next(&mut self, n: usize, display: &Display, panel: usize) {
		let offsetx = 23 + self.offset_x[panel] as u16;
		let offsety = self.offset_y[panel] as u16 + 1;
		let mut doubley = offsety * 2;
		for (x, y) in self.last_dirtypos[panel as usize].drain(..) {
			print!("{} ", termion::cursor::Goto(x as u16, y as u16));
		}
		self.disp_box(offsetx - 1, offsetx + 4, offsety - 1, offsety + 1);
		for &code in
			[display.hold].iter().chain(display.bag_preview[..n].iter())
		{
			if code == 7 {
				doubley += 5;
				continue;
			}
			let mut tmpx = offsetx;
			if code == 3 {
				tmpx += 1;
			}
			self.mini_blockp(tmpx as u32, doubley as u32, code, panel);
			doubley += 5;
		}
	}

	pub fn disp_by_id(&mut self, display: &Display) {
		let panel = match self.id_panel.get(&display.id) {
			Some(&panel) => panel,
			None => {
				eprintln!("Wrong client received");
				return;
			}
		};
		self.disp_by_panel(display, panel);
	}

	pub fn disp_by_panel(&mut self, display: &Display, panel: usize) {
		if panel >= 2 {
			panic!("Only support 2 panels");
		}
		if !self.checksize() {
			return;
		}
		print!("[30m");
		let offsetx = self.offset_x[panel] as u16;
		let offsety = self.offset_y[panel] as u16;
		for i in 0..10 {
			for j in 0..20 {
				self.blockp(
					offsetx + i * 2,
					offsety + 19 - j,
					display.color[j as usize][i as usize],
					0,
				);
			}
		}
		let shadow_block = display.shadow_block.clone();
		let floating_block = display.floating_block.clone();
		let shadow_pos = shadow_block.getpos();
		// show shadow_block first
		print!("[0m");
		for shadow_square in shadow_pos.iter() {
			let x = shadow_square.0 as u16;
			let y = shadow_square.1 as u16;
			if y < 20 {
				self.blockp(
					offsetx + x * 2,
					offsety + 19 - y,
					ID_TO_CHAR[floating_block.code as usize],
					1,
				);
			}
		}
		print!("[30m");
		let tmp_pos = floating_block.getpos();
		for tmp_square in tmp_pos.iter() {
			let x = tmp_square.0 as u16;
			let y = tmp_square.1 as u16;
			if y < 20 {
				self.blockp(
					offsetx + x * 2,
					offsety + 19 - y,
					ID_TO_CHAR[floating_block.code as usize],
					0,
				);
			}
		}
		print!("{}", termion::style::Reset);
		self.disp_info(&display, offsetx, offsety);
		self.disp_hold_next(6, &display, panel);
		self.disp_atk_by_id(display);
	}

	pub fn disp_atk_by_id(&self, display: &Display) {
		let panel = match self.id_panel.get(&display.id) {
			Some(&panel) => panel,
			None => {
				eprintln!("Wrong client sigatk received.");
				return;
			}
		};
		let offsetx = self.offset_x[panel] + 20;
		let offsety = self.offset_y[panel];
		let mut dy = 0;
		for (mut ind, &each_garbage) in display.garbages.iter().enumerate() {
			let mut each_garbage = each_garbage as u16;
			let flag = dy + each_garbage > 20;
			if flag {
				each_garbage = 20 - dy;
			}
			if ind > 4 {
				ind = 4;
			}
			print!("[4{}m", 5 - ind);
			for i in dy..(dy + each_garbage as u16) {
				print!(
					"{} ",
					termion::cursor::Goto(
						offsetx as u16,
						offsety as u16 + (19 - i)
					),
				)
			}
			if flag {
				break;
			}
			dy += each_garbage;
		}
		print!("{}", termion::style::Reset);
		for i in dy..20 {
			print!(
				"{} ",
				termion::cursor::Goto(
					offsetx as u16,
					offsety as u16 + (19 - i)
				),
			)
		}
	}

	pub fn activate(&self) {
		print!(
			"{}{}{}",
			termion::style::Reset,
			termion::clear::All,
			termion::cursor::Hide
		);
		self.disp_mainbox();
	}

	pub fn deactivate(&self) {
		print!(
			"{}{}{}{}",
			termion::style::Reset,
			termion::clear::All,
			termion::cursor::Goto(1, 1),
			termion::cursor::Show,
		);
	}
}

impl Drop for ClientDisplay {
	fn drop(&mut self) {
		self.deactivate();
	}
}
