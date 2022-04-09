use crate::stupid_mcts::SearchTree;
use crate::Thinker;
use tttz_protocol::{Display, KeyType};

use std::collections::VecDeque;

#[derive(Default)]
pub struct SBAi {
	search_tree: Option<SearchTree>,
	rst: bool,
}

impl Thinker for SBAi {
	fn reset(&mut self) {
		self.rst = true;
		self.search_tree = None;
	}

	// TODO: handle garbages
	fn main_think(&mut self, displays: Vec<Display>) -> VecDeque<KeyType> {
		if self.search_tree.is_none() {
			let mut search_tree =
				SearchTree::from_display(displays.into_iter().next().unwrap());
			let ret = search_tree.go_down();
			self.search_tree = Some(search_tree);
			ret
		} else if let Some(search_tree) = self.search_tree.as_mut() {
			search_tree.update(displays.into_iter().next().unwrap())
		} else {
			unreachable!()
		}
	}
}
