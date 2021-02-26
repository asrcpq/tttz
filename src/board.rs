use crate::block::Block;
use crate::srs_data::*;
use crate::random_generator::RandomGenerator;

pub struct Board {
	print_size: (u16, u16),
	color: Vec<u8>,
	ontop: bool,
	tmp_block: Block,
	shadow_block: Block,
	rg: RandomGenerator,
	hold: u8,
}

impl Board {
	fn is_pos_inside(&self, pos: (i32, i32)) -> bool {
		if pos.0 < 0 || pos.1 < 0 {
			return false
		}
		if pos.0 >= 10 || pos.1 >= 40 {
			return false
		}
		true
	}

	pub fn is_pos_vacant(&self, pos: (i32, i32)) -> bool {
		if !self.is_pos_inside(pos) {
			return false
		}
		self.color[pos.0 as usize + pos.1 as usize * 10] == 7
	}

	pub fn new() -> Board {
		let mut rg: RandomGenerator = Default::default();
		Board {
			print_size: (2, 1),
			ontop: true,
			color: vec![7; 10 * 40],
			tmp_block: Block::new(rg.get()),
			shadow_block: Block::new(0),
			rg,
			hold: 7,
		}
	}

	fn movedown1(&mut self) -> bool {
		self.tmp_block.pos.1 += 1;
		if !self.tmp_block.test(self) {
			self.tmp_block.pos.1 -= 1;
			self.hard_drop();
			return false
		}
		true
	}

	pub fn slowdown(&mut self, dy: u16) {
		let first_visible = 21 - BLOCK_HEIGHT[(
			self.tmp_block.code * 4 +
			self.tmp_block.rotation as u8
		) as usize];
		if self.tmp_block.pos.1 < first_visible {
			for _ in self.tmp_block.pos.1..first_visible {
				self.movedown1();
			}
		} else {
			for _ in 0..dy {
				if !self.movedown1() {
					break
				}
			}
		}
	}

	pub fn move1(&mut self, dx: i32) -> bool {
		self.tmp_block.pos.0 -= dx;
		if !self.tmp_block.test(self) {
			self.tmp_block.pos.0 += dx;
			return false
		}
		true
	}

	pub fn move2(&mut self, dx: i32) {
		while self.move1(dx) {}
	}

	pub fn rotate(&mut self, dr: i8) -> bool {
		if self.tmp_block.code == 3 {
			return false
		}
		let revert_block = self.tmp_block.clone();
		self.tmp_block.rotate(dr);
		if !self.ontop {
			let std_pos = self.tmp_block.pos;
			let len = if dr == 2 {
				6
			} else {
				5
			};
			for wkid in 0..len {
				let right_offset = (dr == 1) as i8 * 40;
				let idx = (revert_block.rotation * 10 + right_offset + wkid * 2) as usize;
				let wkd: &Vec<i32> = 
					if dr == 2 {
						&FWKD
					} else if revert_block.code == 0 {
						&IWKD
					} else {
						&WKD
					};
				self.tmp_block.pos.0 = std_pos.0 + wkd[idx];
				self.tmp_block.pos.1 = std_pos.1 + wkd[idx + 1];
				if self.tmp_block.test(self) {
					return true;
				}
			}
			self.tmp_block = revert_block;
			false
		} else {
			if self.ontop {
				self.tmp_block.pos.1 = 0;
			}
			if !self.tmp_block.test(self) {
				self.tmp_block = revert_block;
				return false;
			}
			true
		}
	}

	pub fn hold(&mut self) {
		if self.hold == 7 {
			self.hold = self.tmp_block.code;
			self.tmp_block = Block::new(self.rg.get());
		} else {
			let tmp = self.hold;
			self.hold = self.tmp_block.code;
			self.tmp_block = Block::new(tmp);
		}
		self.ontop = true;
	}

	pub fn soft_drop(&mut self) -> bool {
		self.ontop = false;
		if self.shadow_block.pos.1 == self.tmp_block.pos.1 {
			return false;
		}
		self.tmp_block.pos.1 = self.shadow_block.pos.1;
		true
	}

	pub fn checkline(&mut self, ln: Vec<usize>) {
		let mut elims = Vec::new();
		for each_ln in ln.iter() {
			let mut flag = true;
			for i in each_ln * 10..(each_ln + 1) * 10 {
				if self.color[i] == 7 {
					flag = false;
				}
			}
			if flag {
				elims.push(each_ln);
			}
		}
		if elims.is_empty() {
			return
		}
		let mut movedown = 0;
		for i in (0..40).rev() {
			let mut flag = false;
			for elim in elims.iter() {
				if i == **elim {
					flag = true;
					break
				}
			}
			if flag {
				movedown += 1;
				continue
			}
			if movedown == 0 {
				continue
			}
			for j in 0..10 {
				self.color[(i + movedown) * 10 + j] = 
					self.color[i * 10 + j];
			}
		}
	}

	pub fn hard_drop(&mut self) {
		let tmppos = self.tmp_block.getpos();
		let mut lines_tocheck = Vec::new();
		for i in 0..4 {
			let px = tmppos[i * 2] as usize;
			let py = tmppos[i * 2 + 1] as usize;

			let mut flag = true;
			for l in lines_tocheck.iter() {
				if *l == py {
					flag = false;
				}
			}
			if flag {
				lines_tocheck.push(py);
			}

			self.color[px + py * 10] = self.tmp_block.code;
		}
		self.checkline(lines_tocheck);
		self.ontop = true;
		self.tmp_block = Block::new(self.rg.get());
	}

	pub fn press_down(&mut self) {
		if !self.soft_drop() {
			self.hard_drop();
		}
	}

	pub fn press_up(&mut self) {
		self.soft_drop();
		self.hard_drop();
	}

	fn calc_shadow(&mut self) -> bool {
		// prevent infloop
		self.shadow_block = self.tmp_block.clone();
		loop {
			self.shadow_block.pos.1 += 1;
			if !self.shadow_block.test(self) {
				if self.shadow_block.pos.1 < 20 {
					panic!("Game over is not implemented!");
				} else {
					self.shadow_block.pos.1 -= 1;
					return true
				}
			}
		}
	}

	#[allow(dead_code)]
	fn blockp2(&self, i: u16, mut j: u16, color: u8) {
		if j < 20 { return }
		j -= 20;
		for pi in 0..self.print_size.0 {
			for pj in 0..self.print_size.1 {
				print!(
					"{}[4{}m ",
					termion::cursor::Goto(
						1 + i * self.print_size.0 as u16 + pi,
						1 + j * self.print_size.1 as u16 + pj,
					),
					COLORMAP[color as usize],
				);
			}
		}
	}

	#[allow(dead_code)]
	fn blockp(&self, i: u16, mut j: u16, color: u8, style: u8) {
		if j < 20 { return }
		j -= 20;
		if color == 7 {
			print!(
				"[40m{} {} ",
				termion::cursor::Goto(
					1 + i * self.print_size.0 as u16,
					1 + j * self.print_size.1 as u16,
				),
				termion::cursor::Goto(
					1 + i * self.print_size.0 as u16 + 1,
					1 + j * self.print_size.1 as u16,
				),
			);
			return
		}
		let (ch1, ch2) = if style == 0 {
			('[', ']')
		} else {
			(' ', ' ')
		};
		print!(
			"[4{}m{}{}{}{}",
			COLORMAP[color as usize],
			termion::cursor::Goto(
				1 + i * self.print_size.0 as u16,
				1 + j * self.print_size.1 as u16,
			),
			ch1,
			termion::cursor::Goto(
				1 + i * self.print_size.0 as u16 + 1,
				1 + j * self.print_size.1 as u16,
			),
			ch2,
		);
	}

	pub fn proc(&mut self) {
		self.calc_shadow();
		self.disp();
	}

	fn disp_block(&self, block: &Block, style: u8) {
		let tmp_pos = block.getpos();
		for i in 0..4 {
			let x = tmp_pos[i * 2];
			let y = tmp_pos[i * 2 + 1];
			self.blockp(x, y, block.code, style);
		}
	}

	fn disp_next(&self, n: u16) {
		let offsetx = 1;
		let offsety = 21;
		println!("{}hold: {}", 
			termion::cursor::Goto(
				offsetx,
				offsety,
			),
			ID_TO_CHAR[self.hold as usize],
		);
		for i in 0..n {
			println!("{}{}", 
				termion::cursor::Goto(
					offsetx + i,
					offsety + 1,
				),
				ID_TO_CHAR[self.rg.bag[i as usize] as usize],
			);
		}
	}

	fn disp(&self) {
		for i in 0..10 {
			for j in 20..40 {
				self.blockp(
					i,
					j,
					self.color[i as usize + j as usize * 10],
					0,
				);
			}
		}
		// show shadow_block first
		self.disp_block(&self.shadow_block, 1);
		self.disp_block(&self.tmp_block, 0);
		println!("{}", termion::style::Reset);
		self.disp_next(6);
	}
}
