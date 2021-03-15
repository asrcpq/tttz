use crate::{CodeType, PosType};

// Mini check for J and l: only check center square
lazy_static::lazy_static! {
pub static ref TWIST_MINI_CHECK: [[Vec<(PosType, PosType)>; 4]; 7] = [
	[
		vec![],
		vec![],
		vec![],
		vec![], // I spin does not have mini type
	],
	[
		vec![(1, 1)],
		vec![(1, 1)],
		vec![(1, 0)],
		vec![(0, 1)],
	],
	[
		vec![(1, 1)],
		vec![(1, 1)],
		vec![(1, 0)],
		vec![(0, 1)],
	],
	[
		vec![],
		vec![],
		vec![],
		vec![], // O cannot spin
	],
	[
		vec![(0, 1), (2, 0)],
		vec![(1, 2), (0, 0)],
		vec![(0, 1), (2, 0)],
		vec![(1, 2), (0, 0)],
	],
	[
		vec![(0, 1), (2, 1)],
		vec![(1, 2), (1, 0)],
		vec![(0, 0), (2, 0)],
		vec![(0, 2), (0, 0)],
	],
	[
		vec![(2, 1), (0, 0)],
		vec![(0, 2), (1, 0)],
		vec![(2, 1), (0, 0)],
		vec![(0, 2), (1, 0)],
	],
];

// wall kick pos
// line 1-4: 0->1 to 3->0, 5 attempts
// line 5-8: 0->3 to 3->2
static ref WKD: [Vec<(PosType, PosType)>; 8] = [
	 vec![(1, -1), (0, -1), (0, 0), (1, -3), (0, -3)],
	 vec![(-1, 0), (0, 0), (0, -1), (-1, 2), (0, 2)],
	 vec![(0, 0), (1, 0), (1, 1), (0,-2), (1,-2)],
	 vec![(0, 1), (-1, 1), (-1, 0), (0, 3), (-1, 3)],
	 vec![(0, -1), (1, -1), (1, 0), (0, -3), (1, -3)],
	 vec![(-1, 1), (0, 1), (0, 0), (-1, 3), (0, 3)],
	 vec![(1, 0), (0, 0), (0, 1), (1, -2), (0, -2)],
	 vec![(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)],
];
// I block's WKD
static ref IWKD: [Vec<(PosType, PosType)>; 8] = [
	vec![(2, -2), (0, -2), (3, -2), (0, -3), (3, 0)],
	vec![(-2, 1), (-3, 1), (0, 1), (-3, 3), (0, 0)],
	vec![(1, -1), (3, -1), (0, -1), (3, 0), (0, -3)],
	vec![(-1, 2), (0, 2), (-3, 2), (0, 0), (-3, 3)],
	vec![(1, -2), (0, -2), (3, -2), (0, 0), (3, -3)],
	vec![(-2, 2), (0, -2), (-3, 2), (0, 3), (-3, 0)],
	vec![(2, -1), (3, -1), (0, -1), (3, -3), (0, 0)],
	vec![(-1, 1), (-3, 1), (0, 1), (-3, 0), (0, 3)],
];
// flip wall kick, tetr.io style
// 0->2 to 3->1
static ref FWKD: [Vec<(PosType, PosType)>; 4] = [
	vec![(0, -1), (0, 0), (1, 0), (-1, 0), (1, -1), (-1, -1)],
	vec![(-1, 0), (0, 0), (0, 2), (0, 1), (-1, 2), (-1, 1)],
	vec![(0, 1), (0, 0), (-1, 0), (1, 0), (-1, 1), (1, 1)],
	vec![(1, 0), (0, 0), (0, 2), (0, 1), (1, 2), (1, 1)],
];
}

// assume dr = 1, -1, 2
pub fn kick_iter(
	code: CodeType,
	start: i8,
	dr: i8,
) -> impl Iterator<Item = &'static (PosType, PosType)> {
	if dr == 2 {
		return FWKD[start as usize].iter();
	}
	if code != 0 {
		WKD[(dr == -1) as usize * 4 + start as usize].iter()
	} else {
		IWKD[(dr == -1) as usize * 4 + start as usize].iter()
	}
}
