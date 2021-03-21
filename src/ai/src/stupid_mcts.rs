use tttz_ruleset::CodeType;
use tttz_protocol::{KeyType, Display, Piece};
use tttz_mpboard::Field;
use tttz_libai::{access_floodfill, route_solver};
use tttz_libai::evaluation::{Evaluator, SimpleEvaluator};

use std::collections::{HashMap, VecDeque};

#[derive(Debug)]
struct Node {
	pub field: Field,
	pub active: (CodeType, CodeType),
	pub depth: usize, // preview pointer
	pub simple_evaluator: SimpleEvaluator,

	pub id: u64,
	pub parent_id: u64,

	pub weights: HashMap<Piece, f32>,
	pub visit: i32,
	pub q: f32,
	pub children: HashMap<Piece, u64>,
}

impl Node {
	// root
	pub fn from_display(id: u64, depth: usize, display: &Display) -> Node {
		Node {
			field: Field::from_color(&display.color),
			active: (display.floating_block.code, display.hold),
			depth,
			simple_evaluator: SimpleEvaluator::evaluate_field(&display.color),
			id,
			parent_id: id,
			weights: HashMap::new(),
			visit: 0,
			q: 0.0,
			children: HashMap::new(),
		}
	}

	pub fn create(&mut self, piece: Piece, next: CodeType, id: u64) -> Self {
		let (q, new_field) = self.simple_evaluator.evaluate_piece(&self.field.color, &piece);

		let active = if piece.code == self.active.0 {
			(next, self.active.1)
		} else {
			(next, self.active.0)
		};
		self.children.insert(piece, id);
		let simple_evaluator = SimpleEvaluator::evaluate_field(&new_field.color);
		Node {
			field: new_field,
			active,
			depth: self.depth + 1,
			simple_evaluator,
			id,
			parent_id: self.id,
			weights: HashMap::new(),
			visit: 0,
			q,
			children: HashMap::new(),
		}
	}

	pub fn expand(&mut self) -> bool {
		assert!(self.active.0 != 7);
		if !self.weights.is_empty() {
			return false;
		}

		let mut possible = access_floodfill(&self.field.color, self.active.0);
		if self.active.0 != self.active.1 {
			possible.extend(access_floodfill(&self.field.color, self.active.1));
		}

		for piece in possible.iter() {
			let score = self.simple_evaluator.evaluate_piece(&self.field.color, piece).0;
			self.weights.insert(piece.clone(), score);
		}

		// if self.weights.is_empty() { } // die
		true
	}
}

pub struct SearchTree {
	pub preview: Vec<CodeType>,
	root: u64,
	nodes: HashMap<u64, Node>,
	preview_pointer: usize, // or game step, used to index preview
	alloc_id: u64,

	step: u64,
}

impl SearchTree {
	// initialize game
	pub fn from_display(mut display: Display) -> SearchTree {
		display.hold = display.floating_block.code;
		display.floating_block.code = display.bag_preview[0];
		let mut root = Node::from_display(0, 1, &display);
		root.expand();
		let preview = display.bag_preview.to_vec();
		let mut nodes = HashMap::new();
		nodes.insert(0, root);
		SearchTree {
			preview,
			root: 0,
			nodes,
			preview_pointer: 0,
			alloc_id: 1, // 0 is given to root
			step: 2000,
		}
	}

	// unfortunately, for unknown reason
	// replacing the with GarbageOverflow + PlainDrop reply check has bug
	// check if internal state is corrupted
	pub fn compare_display(&self, display: &Display) -> bool {
		let root_node = self.nodes.get(&self.root).unwrap();
		for (i, &line) in display.color.iter().enumerate() {
			if root_node.field.color[i] != line {
				// eprintln!("{:?}", display.color);
				// eprintln!("{:?}", root_node.field);
				return false
			}
		}
		true
	}

	// also handle garbage flush
	pub fn update(&mut self, display: Display) -> VecDeque<KeyType> {
		// update preview
		// eprintln!("in {:?} self {:?} p:{}", display.bag_preview, self.preview, self.preview_pointer);
		for (i, &code) in display.bag_preview.iter().enumerate() {
			// initialize has seq = 0, after first drop, two pieces are consumed
			// so we should check from 2 at seq = 1
			match self.preview.get(i + self.preview_pointer) {
				None => self.preview.push(code),
				Some(code2) => assert_eq!(code, *code2),
			}
		}

		if !self.compare_display(&display) {
			let mut new_root = Node::from_display(self.alloc_id, self.preview_pointer, &display);
			new_root.expand();
			self.root = self.alloc_id;
			self.nodes = HashMap::new();
			self.nodes.insert(self.alloc_id, new_root);
			self.alloc_id += 1;
		}
		self.go_down()
	}

	// single round of expansion, return final node and q
	fn expand_forward(&mut self, root: u64) -> (u64, f32) {
		const DIE_WEIGHT: f32 = -100f32;
		let mut focus = root;
		let mut depth = self.preview_pointer;
		let q = loop {
			// eprintln!("Walking forward on {}", focus);
			let next_code = match self.preview.get(depth) {
				None => break self.nodes.get(&focus).unwrap().q,
				Some(code) => *code,
			};

			let ret = self.select(focus);
			match ret {
				SelectResult::End(piece) => {
					let node = self.nodes
						.get_mut(&focus)
						.unwrap();
					let mut new_node = node.create(piece, next_code, self.alloc_id);
					assert!(new_node.expand());
					let q = node.q;
					// eprintln!("Insert {}", self.alloc_id);
					self.nodes.insert(self.alloc_id, new_node);
					self.alloc_id += 1;
					break q;
				},
				SelectResult::Node(id) => {
					focus = id;
				}
				SelectResult::Die => {
					break DIE_WEIGHT;
				}
			}
			depth += 1;
		};
		(focus, q)
	}

	// update q
	fn expand_backward(&mut self, end_node: u64, q: f32) {
		// self.debug_print_nodes_recurse(self.root);
		// eprintln!();
		let mut focus = end_node;
		loop {
			focus = self.nodes.get(&focus).unwrap().parent_id;
			if focus == self.root {
				break
			}
			// eprintln!("focus: {}", focus);
			let mut node = self.nodes.get_mut(&focus).unwrap(); //parent always exists
			node.q = (node.q * node.visit as f32 + q) / (node.visit + 1) as f32;
			node.visit += 1;
		}
	}

	pub fn go_down(&mut self) -> VecDeque<KeyType> {
		// let root_node = self.nodes.get(&self.root).unwrap();
		// eprintln!("{:?} {:?}", root_node, self.preview);
		let mut ret = VecDeque::new();
		if self.preview_pointer == 0 {
			ret.push_back(KeyType::Hold);
			self.preview_pointer += 1;
		}
		// rounds of expansion
		for _ in 0..self.step {
			// forward
			let (end_node, q) = self.expand_forward(self.root);
			// bp
			self.expand_backward(end_node, q);
		}
		// find most visited
		ret.extend(self.decide());
		ret.push_back(KeyType::HardDrop);
		self.preview_pointer += 1;
		ret
	}

	fn drop_recurse(&mut self, root: u64) {
		let node = self.nodes.remove(&root).unwrap();
		// eprintln!("drop {}", root);
		for (_piece, &child_id) in &node.children {
			self.drop_recurse(child_id);
		}
	}

	fn decide(&mut self) -> VecDeque<KeyType> {
		let node = self.nodes.get(&self.root).unwrap();
		let root_children = &node.children;
		let mut max_visit = -1;
		let mut best_piece = None;
		let mut best_id = 0;
		for (piece, &id) in root_children {
			if let Some(child) = self.nodes.get(&id) {
				if child.visit > max_visit {
					max_visit = child.visit;
					best_piece = Some(piece.clone());
					best_id = id;
				}
			}
		}

		// preserve best_id, delete other leaves
		let root = self.nodes.remove(&self.root).unwrap();
		for (_piece, &child_id) in &root.children {
			if child_id != best_id {
				self.drop_recurse(child_id);
			}
		}

		let best_piece = best_piece.unwrap();
		let mut ret = VecDeque::new();
		if root.active.0 != best_piece.code {
			ret.push_back(KeyType::Hold);
		}
		self.root = best_id;
		self.nodes.get_mut(&best_id).unwrap().parent_id = best_id;
		ret.extend(route_solver(&root.field.color, &best_piece).unwrap());
		ret
	}

	fn select(&mut self, focus: u64) -> SelectResult {
		const CPUCT: f32 = 1.0;
		let node = match self.nodes.get(&focus) {
			Some(node) => node,
			None => {
				self.debug_print_nodes_recurse(self.root);
				eprintln!();
				panic!("Selecting nonexist node: {}", focus);
			}
		};
		let mut max_u = f32::NEG_INFINITY;
		let mut max_piece = None;
		let mut max_id = 0;
		for (piece, &value) in node.weights.iter() {
			let (q, visit, id) = if let Some(&id) = node.children.get(piece) {
				let node = self.nodes.get(&id).unwrap();
				(node.q, node.visit, id)
			} else {
				(0f32, 0, 0)
			};
			let u = ((node.visit as f32).sqrt() / (1 + visit) as f32) * value * CPUCT + q;
			// let u = q;
			if max_u < u {
				// eprintln!("{} overtake {} at {:?}", u, max_u, piece);
				max_u = u;
				max_piece = Some(piece);
				max_id = id;
			}
		}
		let max_piece = match max_piece {
			Some(piece) => piece.clone(),
			None => return SelectResult::Die,
		};
		if max_id == 0 {
			SelectResult::End(max_piece)
		} else {
			SelectResult::Node(max_id)
		}
	}

	pub fn debug_print_nodes_recurse(&self, id: u64) {
		match self.nodes.get(&id) {
			None => {
				eprint!("?{}?", id);
				return;
			}
			Some(node) => {
				eprint!("{}", id);
				if node.children.is_empty() { return; }
				eprint!("(");
				for (_piece, &child_id) in &node.children {
					self.debug_print_nodes_recurse(child_id);
					eprint!(" ");
				}
				eprint!(")");
			}
		}
	}
}

#[derive(Debug)]
enum SelectResult {
	Die,
	End(Piece),
	Node(u64),
}

#[cfg(test)]
mod test {
	use super::*;
	use tttz_mpboard::Board;
	use tttz_protocol::BoardMsg;

	#[test]
	fn test_mcts1() {
		let mut board: Board = Default::default();
		let display = board.generate_display(0, 0, BoardReply::Ok);
		eprintln!("hold {}, prev {:?}", display.hold, display.bag_preview);
		let mut search_tree = SearchTree::from_display(display);
		let ret = search_tree.go_down();
		for &op in ret.iter() {
			eprintln!("do {:?}", op);
			board.handle_msg(BoardMsg::KeyEvent(op));
		}
		let display = board.generate_display(0, 0, BoardReply::PlainDrop(0));
		let mut ret = search_tree.update(display);
		assert_eq!(ret.pop_back(), Some(KeyType::HardDrop));
	}
}
