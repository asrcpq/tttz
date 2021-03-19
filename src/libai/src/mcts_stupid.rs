use tttz_ruleset::CodeType;
use tttz_mpboard::Field;
use crate::{access_floodfill, route_solver};

use std::collections::{HashMap, HashSet, VecDeque};

struct Node {
	pub field: Field,
	pub active: (CodeType, CodeType),
	pub depth: usize, // preview pointer
	pub gaman: GarbageAttackManager,
	pub simple_evaluator: SimpleEvaluator,

	pub id: u64,
	pub parent: u64,

	pub weights: HashMap<Piece, f32>,
	pub visit: u32,
	pub q: f32,
	pub children: HashMap<Piece, u64>,
}

enum ExpandResult {
	Die,
	Expanded,
	Ok(Node, bool), // true = hold used
}

impl Node {
	// root
	pub fn from_display(id: u64, depth: usize, display: &Display) -> Node {
		Node {
			field: display.color.to_vec(),
			active: (display.floating_block.code, display.hold),
			depth,
			gaman: GarbageAttackManager::from_display(display),
			simple_evaluator: evaluate_field(display.color),
			id,
			parent_id: id,
			weights: HashMap::new(),
			visit: 0,
			q: 0,
			children: HashMap::new(),
		}
	}

	pub fn expand(&mut self) -> ExpandResult {
		assert!(self.active.0 != 7);
		self.visit += 1;
		if !self.weights.is_empty() {
			return None;
		}
		let mut min_score = f32::INFINITY;
		let mut min_piece = None;

		for piece in access_floodfill(&self.field.color, self.active.0).iter() {
			let score = self.simple_evaluator.evaluate_piece(&self.field.color, piece);
			if score < min_score {
				min_score = score;
				min_piece = Some(piece.clone());
			}
			self.weights.insert(piece.clone(), (score, 0));
		}
		let mut holdflag = false;
		for piece in access_floodfill(&self.field.color, self.active.1).iter() {
			let score = self.simple_evaluator.evaluate_piece(&self.field.color, piece);
			if score < min_score {
				min_score = score;
				min_piece = Some(piece.clone());
				holdflag = true;
			}
			self.weights.insert(piece.clone(), (score, 0));
		}

		let mut min_piece = match min_piece {
			Some(p) => p,
			None => return None,
		};
		let twist = self.field.test_twist(&mut min_piece);
		let mut new_field = self.field.clone();
		let lc = new_field.settle_block(min_piece);
		let new_gaman = self.gaman.clone();
		let atk = new_gaman.calc_attack(
			twist,
			lc,
			min_piece.code,
			new_field.height == 0
		);
		Some(Node {
			field: new_field,
			active: self.active, // to change
			depth: self.depth + 1,
			gaman: new_gaman,
			simple_evaluator: evaluate_field(&new_field.color),
			id: 0, // to change
			parent_id: self.id,
			weights: HashMap::new(),
			visit: 0,
			q: atk - min_score,
			children: HashMap::new(),
		}, holdflag)
	}
}

pub struct SearchTree {
	pub preview: Vec<CodeType>,
	root: u64,
	nodes: HashMap<u64, Node>,
	preview_pointer: usize, // or game step, used to index preview
	alloc_id: u64,
}

impl SearchTree {
	// initialize game
	pub fn from_display(mut display: Display) -> SearchTree {
		display.hold = display.floating_block.code;
		display.floating_block.code = display.bag_preview[0];
		let root = Node::from_display(0, 1, &Display);
		let preview = display.bag_preview.to_vec();
		let mut nodes = HashMap::new();
		nodes.insert(0, root);
		SearchTree {
			preview,
			root: 0,
			nodes,
			child_map: HashMap::new(),
			preview_pointer: 1, // zero is hold
			alloc_id: 1, // 0 is given to root
		}
	}

	// handle garbage flush
	pub fn update(&mut self, display: Display) -> VecDeque<KeyType> {
		// update preview
		for (i, code) in display.bag_preview.iter().enumerate() {
			// initialize has seq = 0, after first drop, two pieces are consumed
			// so we should check from 2 at seq = 1
			match self.preview.get_mut(i + display.seq + 1) {
				None => self.preview.push(code),
				Some(code2) => assert_eq!(code, code2),
			}
		}

		let flush_flag = match display.board_reply {
			BoardReply::GarbageOverflow => true,
			BoardReply::PlainDrop(x) if x > 0 => true,
			_ => false,
		}
		if flush_flag {
			self.child_map = HashMap::new();

			let node = Node::from_display(self.alloc_id, self.preview_pointer, display);
			self.nodes = HashMap::new();
			self.nodes.insert(self.alloc_id, node);
			self.alloc_id += 1;
		}
		self.go_down()
	}

	pub fn go_down(&mut self, steps: u32) -> VecDeque<KeyType> {
		const DEATH_Q: f32 = -100.0;
		let ret = VecDeque::new();
		if self.preview_pointer = 0 {
			ret.push(KeyType::Hold);
			self.preview_pointer += 1;
		}
		let mut focus = self.root;
		// rounds of expansion
		for _ in 0..steps {
			let mut depth = self.preview_pointer;
			// forward
			let ret = loop {
				let next_code = match self.preview.get(depth) {
					None => break self.get(focus).unwrap().q,
					Some(code) => code,
				}
				let ret = self.get_mut(focus).unwrap().expand(self.preview[self.depth]);
				match ret {
					ExpandResult::Ok((mut node, holdflag)) => {
						let q = node.q;
						node.id = self.alloc_id;
						if holdflag {
							node.active.1 = node.active.0;
						}
						self.preview_pointer += 1;
						node.active.0 = self.preview.get(depth).unwrap_or(7);
						self.nodes.insert(self.alloc_id, node);
						self.alloc_id += 1;
						break node.q;
					}
					ExpandResult::Die => break DEATH_Q;
					ExpandResult::Expanded => {
						let focus = self.select(focus);
					}
				}
				depth += 1;
			}

			// bp

		}
		// find most visited


		self.preview_pointer += 1;
	}

	fn select(&mut self, focus: u64) -> u64 {
		const CPUCT: f32 = 2.5;
		for (piece, (value, visit)) in self.weights.iter_mut() {
			((self.visit as f32).sqrt() / (1 + visit as f32)) * value * CPUCT * 
		}
	}
}
