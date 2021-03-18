use crate::*;

use tttz_mpboard::Field;
use tttz_ruleset::CodeType;
use tttz_protocol::Piece;

// TODO: use bit to save memory
pub fn pc_solver_recurse<'a>(seq: impl Iterator<Item = &'a CodeType> + Clone, field: Field)
	-> Option<Vec<Piece>>
{
	let mut next = seq.clone();
	let code = match next.next() {
		Some(&code) => code,
		None => if field.height == 0 {
			return Some(Vec::new())
		} else {
			return None
		}
	};
	// eprintln!("proc code {}", code);
	for piece in access_floodfill(&field.color, code) {
		// eprintln!("try pos {:?}", piece);
		let mut field = field.clone();
		field.settle_block(&piece);
		if field.height > 4 {
			continue
		}
		match pc_solver_recurse(next.clone(), field) {
			None => {},
			Some(mut vec) => {
				vec.push(piece);
				return Some(vec)
			}
		}
	}
	// eprintln!("all fail");
	None
}

pub fn pc_solver_blank(seq: Vec<CodeType>) -> Option<Vec<Piece>> {
	// at most 7 lines, for I spin
	let new_field = Field::from_color(&vec![[b' '; 10]; 7]);
	pc_solver_recurse(seq.iter(), new_field)
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
		} assert!(pc_solver_recurse(vec![6].iter(), Field::from_color(&color)).is_some())
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
		assert!(pc_solver_recurse(vec![0, 2, 3, 0, 6, 4].iter(), Field::from_color(&color)).is_some())
	}

	#[test]
	fn test_pc_all_i() {
		assert!(pc_solver_blank(vec![0; 10]).is_some());
	}

	#[test]
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

	#[test]
	#[ignore]
	fn test_pc_2() {
		// zjstiloizs
		assert!(pc_solver_blank(vec![6, 1, 5, 4, 0, 2, 3, 0, 6, 4]).is_some());
	}
}
