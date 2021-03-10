lazy_static::lazy_static! {
// each line has four groups of two pairs of points of twist check
pub static ref TWIST_MINI_CHECK: Vec<i32> = vec![
	0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // I spin does not have mini type
	1, 0, 2, 0, 1, 1, 1, 2, 0, 1, 1, 1, 0, 0, 0, 1,
	0, 0, 1, 0, 1, 0, 1, 1, 1, 1, 2, 1, 0, 1, 0, 2,
	0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // O cannot spin
	0, 0, 2, 1, 1, 0, 0, 2, 0, 0, 2, 1, 1, 0, 0, 2,
	0, 0, 2, 0, 1, 0, 1, 2, 0, 1, 2, 1, 0, 0, 0, 2,
	2, 0, 0, 1, 0, 0, 1, 2, 2, 0, 0, 1, 0, 0, 1, 2,
];

// standard rotation pos
// each line is for a type of block, 4 pairs of pos(left up) indicates 4 directions
// each pos is the difference to first pair
pub static ref SRP: Vec<i32> = vec![
	0, 0, 2, -1, 0, 1, 1, -1,
	0, 0, 1, 0, 0, 1, 0, 0,
	0, 0, 1, 0, 0, 1, 0, 0,
	0, 0, 0, 0, 0, 0, 0, 0,
	0, 0, 1, 0, 0, 1, 0, 0,
	0, 0, 1, 0, 0, 1, 0, 0,
	0, 0, 1, 0, 0, 1, 0, 0,
];

// wall kick pos
// line 1-4: 0->1 to 3->0, 5 attempts
// line 5-8: 0->3 to 3->2
pub static ref WKD: Vec<i32> = vec![
	 0, 0, -1, 0, -1,-1, 0, 2, -1, 2,
	 0, 0,  1, 0,  1, 1, 0,-2,  1,-2,
	 0, 0,  1, 0,  1,-1, 0, 2,  1, 2,
	 0, 0, -1, 0, -1, 1, 0,-2, -1,-2,
	 0, 0,  1, 0,  1,-1, 0, 2,  1, 2,
	 0, 0,  1, 0,  1, 1, 0,-2,  1,-2,
	 0, 0, -1, 0, -1,-1, 0, 2, -1, 2,
	 0, 0, -1, 0, -1, 1, 0,-2, -1,-2,
];
// I block's WKD
pub static ref IWKD: Vec<i32> = vec![
	0, 0, -2, 0,  1, 0, -2, 1,  1,-2,
	0, 0, -1, 0,  2, 0, -1,-2,  2, 1,
	0, 0,  2, 0, -1, 0,  2,-1, -1, 2,
	0, 0,  1, 0, -2, 0,  1, 2, -2,-1,
	0, 0, -1, 0,  2, 0, -1,-2,  2, 1,
	0, 0,  2, 0, -1, 0,  2,-1, -1, 2,
	0, 0,  1, 0, -2, 0,  1, 2, -2,-1,
	0, 0, -2, 0,  1, 0, -2, 1,  1,-2,
];
// flip wall kick, tetr.io style
// 0->2 to 3->1
pub static ref FWKD: Vec<i32> = vec![
	0, 0, 0, -1,  1, -1, -1, -1,  1,  0, -1,  0,
	0, 0, 1,  0,  1, -2,  1, -1,  0, -2,  0, -1,
	0, 0, 0,  1, -1,  1,  1,  1, -1,  0,  1,  0,
	0, 0, -1, 0, -1, -2, -1, -1,  0, -2,  0, -1,
];
}
