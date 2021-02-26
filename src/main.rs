extern crate termion;
extern crate rand;
extern crate lazy_static;

use termion::event::{Key, Event};
use termion::input::{TermRead, MouseTerminal};
use termion::raw::IntoRawMode;
use std::io::{Write, stdout, stdin};

mod block;
mod board;
mod srs_data;
mod random_generator;
use board::Board;

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
			Event::Key(Key::Char('J')) => {b.slowdown(1);},
			Event::Key(Key::Char('K')) => {b.slowdown(5);},
			Event::Key(Key::Char('z')) => {b.rotate(-1);},
			Event::Key(Key::Char('x')) => {b.rotate(1);},
			Event::Key(Key::Char('d')) => {b.rotate(2);},
			Event::Key(Key::Char(' ')) => {b.hold();},
			_ => {}
		}
		b.proc();
		stdout.flush().unwrap();
	}
	write!(stdout, "[0;0m{}{}", termion::clear::All, termion::cursor::Show).unwrap();
}
