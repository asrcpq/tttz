use tttz_libai::*;
use tttz_mpboard::Field;

fn main() {
	// zjstiloizs
	let ret = pc_solver_blank(vec![6, 1, 5, 4, 0, 2, 3, 0, 6, 4]).unwrap();
	for each_piece in ret.iter().rev() {
		eprintln!("{:?}", each_piece);
	}
}

