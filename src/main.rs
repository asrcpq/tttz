extern crate termion;
extern crate rand;
extern crate lazy_static;

use termion::event::{Key, Event, MouseEvent};
use termion::input::{TermRead, MouseTerminal};
use termion::raw::IntoRawMode;
use std::io::{Write, stdout, stdin};
use rand::{thread_rng, Rng};

// block pos table
// each two lines are 4 groups, each group is a block in certaion direction
// each group has four pairs, each pair is a pos of a group
lazy_static::lazy_static! {

pub static ref BPT: Vec<i32> = vec![
	0, 0, 1, 0, 2, 0, 3, 0, 0, 0, 0, 1, 0, 2, 0, 3,
	0, 0, 1, 0, 2, 0, 3, 0, 0, 0, 0, 1, 0, 2, 0, 3,
	0, 0, 0, 1, 1, 1, 2, 1, 0, 0, 1, 0, 0, 1, 0, 2,
	0, 0, 1, 0, 2, 0, 2, 1, 1, 0, 1, 1, 1, 2, 0, 2,
	2, 0, 2, 1, 1, 1, 0, 1, 0, 0, 0, 1, 0, 2, 1, 2,
	0, 0, 1, 0, 2, 0, 0, 1, 0, 0, 1, 0, 1, 1, 1, 2,
	0, 0, 0, 1, 1, 0, 1, 1, 0, 0, 0, 1, 1, 0, 1, 1,
	0, 0, 0, 1, 1, 0, 1, 1, 0, 0, 0, 1, 1, 0, 1, 1,
	1, 0, 2, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 1, 2,
	1, 0, 2, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 1, 2,
	1, 0, 0, 1, 1, 1, 2, 1, 0, 0, 0, 1, 0, 2, 1, 1,
	0, 0, 1, 0, 2, 0, 1, 1, 1, 0, 1, 1, 1, 2, 0, 1,
	0, 0, 1, 0, 1, 1, 2, 1, 1, 0, 1, 1, 0, 1, 0, 2,
	0, 0, 1, 0, 1, 1, 2, 1, 1, 0, 1, 1, 0, 1, 0, 2,
];

pub static ref COLORMAP: Vec<u8> = vec![6, 4, 7, 3, 2, 5, 1, 0];

// standard rotation pos
// each line is for a type of block, 4 pairs of pos(left up) indicates 4 directions
// each pos is the difference to first pair
pub static ref SRP: Vec<i32> = vec![
	0, 0, 2, -1, 0, 1, 1, -1,
	0, 0, 1, 0, 0, 1, 0, 0,
	0, 0, 1, 0, 0, 1, 0, 0,
	0, 0, 0, 0, 0, 0, 0, 0,
	0, 0, 1, 0, 0, 1, 0, 0,
	0, 0, 1, 0, 0, 1, 0, 0,
	0, 0, 1, 0, 0, 1, 0, 0,
];

// wall kick pos
// line 1-4: 0->1 to 3->0, 5 attempts
// line 5-8: 0->3 to 3->2
pub static ref WKD: Vec<i32> = vec![
	 0, 0, -1, 0, -1, 1, 0,-2, -1,-2,
	 0, 0,  1, 0,  1,-1, 0, 2,  1, 2,
	 0, 0,  1, 0,  1, 1, 0,-2,  1,-2,
	 0, 0, -1, 0, -1,-1, 0, 2, -1, 2,
	 0, 0,  1, 0,  1, 1, 0,-2,  1,-2,
	 0, 0,  1, 0,  1,-1, 0, 2,  1, 2,
	 0, 0, -1, 0, -1, 1, 0,-2, -1,-2,
	 0, 0, -1, 0, -1,-1, 0, 2, -1, 2,
];
// I block's WKD
pub static ref IWKD: Vec<i32> = vec![
	0, 0, -2, 0,  1, 0, -2,-1,  1, 2,
	0, 0, -1, 0,  2, 0, -1, 2,  2,-1,
	0, 0,  2, 0, -1, 0,  2, 1, -1,-2,
	0, 0,  1, 0, -2, 0,  1,-2, -2, 1,
	0, 0, -1, 0,  2, 0, -1, 2,  2,-1,
	0, 0,  2, 0, -1, 0,  2, 1, -1,-2,
	0, 0,  1, 0, -2, 0,  1,-2, -2, 1,
	0, 0, -2, 0,  1, 0, -2,-1,  1, 2,
];
// flip wall kick, tetr.io style
// 0->2 to 3->1
pub static ref FWKD: Vec<i32> = vec![
	0, 0, 0, 1, 1, 1, -1, 1, 1, 0, -1, 0,
	0, 0, 1, 0, 1, 2, 1, 1, 0, 2, 0, 1,
	0, 0, 0, -1, -1, -1, 1, -1, -1, 0, 1, 0,
	0, 0, -1, 0, -1, 2, -1, 1, 0, 2, 0, 1,
];

}

// clone is used when revert rotation test
#[derive(Clone)]
struct Block {
	pub code: u8,
	pub pos: (i32, i32),
	pub rotation: i8,
}

impl Block {
	pub fn rotate(&mut self, dr: i8) {
		let old_rot = self.rotation;
		self.rotation += dr;
		while self.rotation < 0 {
			self.rotation += 4;
		}
		while self.rotation >= 4 {
			self.rotation -= 4;
		}
		let idx_old = (self.code * 8 + old_rot as u8 * 2) as usize;
		let idx = (self.code * 8 + self.rotation as u8 * 2) as usize;
		self.pos.0 -= SRP[idx_old];
		self.pos.1 -= SRP[idx_old + 1];
		self.pos.0 += SRP[idx];
		self.pos.1 += SRP[idx + 1];
	}

	pub fn initial_pos(code: u8) -> i32 {
		match code {
			3 => 5,
			0 => 3,
			_ => 4,
		}
	}

	pub fn new(code: u8) -> Block {
		Block {
			code,
			pos: (Block::initial_pos(code), 0),
			rotation: 0,
		}
	}

	pub fn getpos(&self) -> [u16; 8] {
		let mut ret = [0u16; 8];
		for block_id in 0..4 {
			let tmp = self.code * 32 + self.rotation as u8 * 8  + block_id * 2;
			let px = self.pos.0 + BPT[tmp as usize];
			let py = self.pos.1 + BPT[tmp as usize + 1];
			ret[block_id as usize * 2] = px as u16;
			ret[block_id as usize * 2 + 1] = py as u16;
		}
		ret
	}

	pub fn test(&self, board: &Board) -> bool {
		for block_id in 0..4 {
			let tmp = self.code * 32 + self.rotation as u8 * 8  + block_id * 2;
			let px = self.pos.0 + BPT[tmp as usize];
			let py = self.pos.1 + BPT[tmp as usize + 1];
			if !board.is_pos_vacant((px, py)) {
				return false
			}
		}
		true
	}
}

struct Board {
	print_size: (u16, u16),
	color: Vec<u8>,
	ontop: bool,
	tmp_block: Block,
	shadow_block: Block,
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

	fn is_pos_vacant(&self, pos: (i32, i32)) -> bool {
		if !self.is_pos_inside(pos) {
			return false
		}
		self.color[pos.0 as usize + pos.1 as usize * 10] == 7
	}

	pub fn new() -> Board {
		Board {
			print_size: (2, 1),
			ontop: true,
			color: vec![7; 10 * 40],
			tmp_block: Block::new(0),
			shadow_block: Block::new(0),
		}
	}

	pub fn move1(&mut self, dx: i32) -> bool {
		self.tmp_block.pos.0 -= dx;
		if !self.tmp_block.test(self) {
			self.tmp_block.pos.0 += dx;
			return false
		}
		return true
	}

	pub fn move2(&mut self, dx: i32) {
		while self.move1(dx) {;}
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
			return false;
		} else {
			if self.ontop {
				self.tmp_block.pos.1 = 0;
			}
			if !self.tmp_block.test(self) {
				self.tmp_block = revert_block;
				return false;
			}
			return true;
		}
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
		self.tmp_block = Block::new(thread_rng().gen_range(0..7));
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
					return false
				} else {
					self.shadow_block.pos.1 -= 1;
					return true
				}
			}
		}
		false
	}

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

	fn blockp(&self, i: u16, mut j: u16, color: u8) {
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
		print!(
			"[4{}m{}[{}]",
			COLORMAP[color as usize],
			termion::cursor::Goto(
				1 + i * self.print_size.0 as u16,
				1 + j * self.print_size.1 as u16,
			),
			termion::cursor::Goto(
				1 + i * self.print_size.0 as u16 + 1,
				1 + j * self.print_size.1 as u16,
			),
		);
	}

	pub fn proc(&mut self) {
		self.calc_shadow();
		self.disp();
	}

	fn disp_block(&self, block: &Block) {
		let tmp_pos = block.getpos();
		for i in 0..4 {
			let x = tmp_pos[i * 2];
			let y = tmp_pos[i * 2 + 1];
			self.blockp(x, y, block.code);
		}
	}

	fn disp(&self) {
		let mut iter = self.color.iter();
		for i in 0..10 {
			for j in 20..40 {
				self.blockp(i, j, self.color[i as usize + j as usize * 10]);
			}
		}
		self.disp_block(&self.tmp_block);
		self.disp_block(&self.shadow_block);
	}
}

fn main() {
	let stdin = stdin();
	let mut stdout = MouseTerminal::from(stdout().into_raw_mode().unwrap());

	write!(stdout, "{}{}", termion::clear::All, termion::cursor::Hide).unwrap();
	stdout.flush().unwrap();

	let mut b = Board::new();

	for c in stdin.events() {
		let evt = c.unwrap();
		match evt {
			Event::Key(Key::Char('q')) => break,
			Event::Key(Key::Char('h')) => {b.move1(1);},
			Event::Key(Key::Char('H')) => {b.move2(1);},
			Event::Key(Key::Char('l')) => {b.move1(-1);},
			Event::Key(Key::Char('L')) => {b.move2(-1);},
			Event::Key(Key::Char('k')) => {b.press_up();},
			Event::Key(Key::Char('j')) => {b.press_down();},
			Event::Key(Key::Char('z')) => {b.rotate(-1);},
			Event::Key(Key::Char('x')) => {b.rotate(1);},
			Event::Key(Key::Char('d')) => {b.rotate(2);},
			// Event::Key(Key::Char('z')) => {eprintln!("{}", b.rotate(-1));},
			// Event::Key(Key::Char('x')) => {eprintln!("{}", b.rotate(1));},
			// Event::Key(Key::Char('d')) => {eprintln!("{}", b.rotate(2));},
			_ => {}
		}
		b.proc();
		stdout.flush().unwrap();
	}
	write!(stdout, "[0;0m{}{}", termion::clear::All, termion::cursor::Show).unwrap();
}
