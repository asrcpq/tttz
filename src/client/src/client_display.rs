extern crate termion;
use termion::raw::IntoRawMode;
use std::io::{Read, Write, StdoutLock};

extern crate mpboard;
use mpboard::display::Display;
use mpboard::srs_data::*;

#[derive(Default)]
pub struct ClientDisplay {}

impl ClientDisplay {
	pub fn new() -> ClientDisplay {
		// goto raw mode after ok
		print!("{}{}", termion::clear::All, termion::cursor::Hide);
		std::io::stdout().flush().unwrap();
		Default::default()
	}
	
	fn blockp(&mut self, i: u8, mut j: u8, color: u8, style: u8) {
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
	
	fn disp_info(&mut self, n: u8, display: &Display, mut offsetx: u8, mut offsety: u8) {
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
		for i in 0..n {
			print!("{}{}",
				termion::cursor::Goto(
					(offsetx + i) as u16,
					(offsety + 1) as u16,
				),
				ID_TO_CHAR[display.bag_preview[i as usize] as usize],
			);
		}
	}
	
	pub fn disp(&mut self, display: Display, offsetx: u8, offsety: u8) {
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
		self.disp_info(6, &display, offsetx, offsety);
		self.disp_atk(display.pending_attack, offsetx, offsety);
	}
	
	pub fn disp_atk(&mut self, atk: u32, mut offsetx: u8, offsety: u8) {
		offsetx += 24;
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
