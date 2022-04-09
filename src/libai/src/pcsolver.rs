use crate::*;

use tttz_mpboard::Field;
use tttz_protocol::Piece;
use tttz_ruleset::CodeType;

pub fn hold_seqgen(
	mut current: (CodeType, CodeType),
	preview: &[CodeType],
) -> impl Iterator<Item = Vec<CodeType>> {
	let output_len = preview.len() + 1;
	let t = 2u128.pow(output_len as u32);
	let mut seq = Vec::new();
	for x in 0..t {
		let mut xx = x;
		let mut result = Vec::new();
		for i in 0..output_len {
			let hold = xx % 2;
			if hold == 1 {
				result.push(current.0);
				current.0 = current.1;
				current.1 = *preview.get(i).unwrap_or(&0);
			} else {
				result.push(current.1);
				current.1 = *preview.get(i).unwrap_or(&0);
			}
			xx /= 2;
		}
		seq.push(result);
	}
	seq.into_iter()
}

// TODO: support hold
pub fn pc_solver_recurse<'a>(
	mut seq: impl Iterator<Item = &'a CodeType> + Clone,
	field: Field,
	remain_lc: i32,
) -> Option<Vec<Piece>> {
	let code = match seq.next() {
		Some(&code) => code,
		None => {
			if field.height == 0 {
				return Some(Vec::new());
			} else {
				return None;
			}
		}
	};
	// eprintln!("proc code {}", code);
	for piece in access_floodfill(&field.color, code) {
		// eprintln!("try pos {:?}", piece);
		let mut field = field.clone();
		let lc = field.settle_block(&piece) as i32;
		let new_remain_lc = remain_lc - lc;
		if field.height > new_remain_lc {
			continue;
		}
		match pc_solver_recurse(seq.clone(), field, new_remain_lc) {
			None => {}
			Some(mut vec) => {
				vec.push(piece);
				return Some(vec);
			}
		}
	}
	// eprintln!("all fail");
	None
}

pub fn pc_solver_blank(seq: Vec<CodeType>) -> Option<Vec<Piece>> {
	// at most 7 lines, for I spin
	let new_field = Field::from_color(&[[b' '; 10]; 7]);
	pc_solver_recurse(seq.iter(), new_field, 4)
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn test_pc_recurse1() {
		let mut color = vec![[b' '; 10]; 7];
		color[0][0] = b'i';
		color[1][2] = b'i';
		for i in 3..10 {
			color[0][i] = b'i';
			color[1][i] = b'i';
		}
		assert!(
			pc_solver_recurse(vec![6].iter(), Field::from_color(&color), 4)
				.is_some()
		)
	}

	#[test]
	fn test_pc_recurse2() {
		let mut color = vec![[b'i'; 10]; 4];
		let color2 = vec![[b' '; 10]; 3];
		color.extend(color2);
		for i in 5..10 {
			color[0][i] = b' ';
		}
		for i in 4..10 {
			color[1][i] = b' ';
		}
		for i in 3..10 {
			color[2][i] = b' ';
		}
		for i in 4..10 {
			color[3][i] = b' ';
		}
		assert!(pc_solver_recurse(
			vec![0, 2, 3, 0, 6, 4].iter(),
			Field::from_color(&color),
			4,
		)
		.is_some())
	}

	#[test]
	fn test_pc_recurse3() {
		let mut color = vec![[b'i'; 10]; 4];
		let color2 = vec![[b' '; 10]; 3];
		color.extend(color2);
		for i in 0..8 {
			color[0][i] = b' ';
		}
		for i in 0..7 {
			color[1][i] = b' ';
		}
		for i in 0..5 {
			color[2][i] = b' ';
		}
		for i in 0..4 {
			color[3][i] = b' ';
		}
		// o/t + s i z 3/5 4 0 6
		for seq in hold_seqgen((3, 5), &vec![4, 0, 6]) {
			eprintln!("test {:?}", seq);
			assert!(pc_solver_recurse(
				seq.iter(),
				Field::from_color(&color),
				4,
			)
			.is_none());
		}
	}

	#[test]
	#[ignore]
	fn test_pc_all_i() {
		assert!(pc_solver_blank(vec![0; 10]).is_some());
	}

	#[test]
	#[ignore]
	fn test_pc_all_o() {
		assert!(pc_solver_blank(vec![3; 10]).is_some());
	}

	#[test]
	#[ignore]
	fn test_pc_all_j() {
		assert!(pc_solver_blank(vec![1; 10]).is_some());
	}

	#[test]
	#[ignore]
	fn test_pc_all_z() {
		assert!(pc_solver_blank(vec![6; 10]).is_none());
	}
}
