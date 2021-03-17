use tttz_mpboard::Field;
use tttz_protocol::{Piece, KeyType};
use tttz_ruleset::*;
use crate::utils::*;

use std::collections::{HashSet, HashMap, VecDeque};

// floodfill without route tracing
// only consider blocks fully inside 10x20 visible region
pub fn access_floodfill(color: &[[u8; 10]], code: CodeType) -> Vec<Piece> {
	let heights = get_heights(color);
	let mut queue: VecDeque<Piece> = VecDeque::new();
	let mut possible = HashSet::new();
	let mut sound = HashSet::new();
	let field = Field::from_color(color);
	for rotation in 0..4 {
		for &pos in convolve_height(&heights, code, rotation).0.iter() {
			let p = Piece {
				pos,
				code,
				rotation,
			};
			if !field.test(&p) {
				continue
			}
			queue.push_back(p.clone());
			possible.insert(p.clone());
			sound.insert(p);
		}
	}
	while let Some(mut piece) = queue.pop_front() {
		// possible op H, L, Z, X, D, J, K(sound)
		piece.pos.0 += 1;
		if field.test(&piece) {
			if possible.insert(piece.clone()) {
				queue.push_back(piece.clone());
			}
		}
		piece.pos.0 -= 2;
		if field.test(&piece) {
			if possible.insert(piece.clone()) {
				queue.push_back(piece.clone());
			}
		}
		piece.pos.0 += 1;
		let revert_piece = piece;
		for rot in [-1, 1, 2].iter() {
			let mut piece = revert_piece.clone();
			if field.rotate(&mut piece, *rot) != 0 {
				if possible.insert(piece.clone()) {
					queue.push_back(piece);
				}
			}
		}
		let mut piece = revert_piece.clone();
		loop {
			piece.pos.1 -= 1;
			if !field.test(&piece) {
				piece.pos.1 += 1;
				break
			}
		}
		if possible.insert(piece.clone()) {
			queue.push_back(piece.clone());
		}
		// sound insert should always be executed!
		sound.insert(piece);
	}
	sound.into_iter().collect()
}

#[derive(Default)]
struct Router {
	nodes: bimap::BiMap::<i32, Piece>,
	id_alloc: i32,
	edges: HashMap::<i32, (i32, KeyType)>,
}

impl Router {
	pub fn add_edge(&mut self, piece: Piece, from: &Piece, key: KeyType) -> bool {
		debug_assert!(!self.nodes.contains_left(&self.id_alloc));
		eprintln!("adding {:?}", piece.pos);
		if self.nodes.contains_right(&piece) {
			return false;
		}
		let ret = self.nodes.insert(self.id_alloc, piece);
		debug_assert_eq!(ret, bimap::Overwritten::Neither);
		self.edges.insert(
			self.id_alloc, (
				*self.nodes.get_by_right(from).unwrap(),
				key,
			),
		);
		self.id_alloc += 1;
		true
	}

	pub fn add_root(&mut self, piece: Piece) -> bool {
		self.nodes.insert(
			-10 - piece.pos.0 as i32 - 10 * piece.rotation as i32,
			piece,
		) == bimap::Overwritten::Neither
	}

	pub fn traceroute(&self, piece: &Piece) -> Option<VecDeque<KeyType>> {
		let mut ret = Vec::new();
		match self.nodes.get_by_right(piece) {
			None => return None,
			Some(id) => {
				let mut id = *id;
				while id >= 0 {
					let pointer = self.edges.get(&id).unwrap();
					id = pointer.0;
					ret.push(pointer.1);
				}
				let piece = self.nodes.get_by_left(&id).unwrap();
				let gkp = GenerateKeyParam {
					hold_swap: false,
					code: piece.code,
					rotation: piece.rotation,
					post_key: KeyType::Nothing,
					dx: piece.pos.0,
				};
				let mut first = generate_keys(gkp);
				first.extend(ret.into_iter().rev());
				return Some(first)
			}
		}
	}
}

pub fn route_solver(color: &[[u8; 10]], piece_query: &Piece) -> Option<VecDeque<KeyType>> {
	let code = piece_query.code;
	let heights = get_heights(color);
	let mut queue: VecDeque<Piece> = VecDeque::new();
	let mut router: Router = Default::default();
	let mut sound = HashSet::new();
	let field = Field::from_color(color);
	for rotation in 0..4 {
		for &pos in convolve_height(&heights, code, rotation).0.iter() {
			let p = Piece {
				pos,
				code,
				rotation,
			};
			if !field.test(&p) {
				continue
			}
			queue.push_back(p.clone());
			router.add_root(p.clone());
			if *piece_query == p {
				return router.traceroute(&p);
			}
			sound.insert(p);
		}
	}
	while let Some(revert_piece) = queue.pop_front() {
		// can this be done earlier?
		if *piece_query == revert_piece {
			return router.traceroute(&revert_piece)
		}
		let mut piece = revert_piece.clone();
		// possible op H, L, Z, X, D, J, K(sound)
		piece.pos.0 += 1;
		if field.test(&piece) {
			if router.add_edge(piece.clone(), &revert_piece, KeyType::Right) {
				queue.push_back(piece.clone());
			}
		}
		piece.pos.0 -= 2;
		if field.test(&piece) {
			if router.add_edge(piece.clone(), &revert_piece, KeyType::Left) {
				queue.push_back(piece.clone());
			}
		}
		piece.pos.0 += 1;
		for rot in [-1, 1, 2].iter() {
			let mut piece = revert_piece.clone();
			if field.rotate(&mut piece, *rot) != 0 {
				if router.add_edge(piece.clone(), &revert_piece, match rot {
					1 => KeyType::Rotate,
					-1 => KeyType::RotateReverse,
					2 => KeyType::RotateFlip,
					_ => unreachable!(),
				}) {
					queue.push_back(piece);
				}
			}
		}
		loop {
			piece.pos.1 -= 1;
			if !field.test(&piece) {
				piece.pos.1 += 1;
				break
			}
		}
		if router.add_edge(piece.clone(), &revert_piece, KeyType::SonicDrop) {
			queue.push_back(piece.clone());
		}
		// sound insert should always be executed!
		sound.insert(piece);
	}
	None
}

#[cfg(test)]
mod test {
	use super::*;

	fn generate_color() -> Vec<[u8; 10]> {
		let mut color = vec![[b' '; 10]; 20];
		color[2] = [b'i'; 10];
		color[2][8] = b' ';
		color[2][9] = b' ';
		color[5] = [b'i'; 10];
		color[5][0] = b' ';
		color[5][1] = b' ';
		color
	}

	#[test]
	fn count() {
		let color = generate_color();
		let ret = access_floodfill(&color, 3);
		assert_eq!(ret.len(), (9 + 8 + 8) * 4);
	}

	#[test]
	fn test_route_solver() {
		let color = generate_color();
		let ret = access_floodfill(&color, 3);
		for each_piece in ret.iter() {
			eprintln!("Solving {:?}", each_piece);
			assert!(route_solver(&color, each_piece).is_some());
		}
	}
}
