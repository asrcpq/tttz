use crate::utils::*;
use crate::*;

use tttz_mpboard::Field;
use tttz_ruleset::CodeType;

fn pc_solver_recurse(seq: impl Iterator<Item = &CodeType>, field: &Vec<[u8; 10]>) {
	for piece in access_floodfill(field, *seq) {
		let new_field = Field::from_color(field);
	}
}

pub fn pc_solver(seq: [u8; 10]) {
	let field = vec![[0; 10]; 7];
	pc_solver_recurse(&seq, &field);
}
