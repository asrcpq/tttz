// attacks 10x
pub const ATTACK_B2B_INC: u32 = 5;
pub const ATTACK_COMBO_INC: u32 = 2;
pub const ATTACK_BASE: [u32; 4] = [5, 10, 20, 40];
pub const ATTACK_BASE_TWIST_MULTIPLIER: [u32; 14] = [
	10, 15, 15, 10, 20, 20, 20, // mini
	15, 20, 20, 10, 30, 30, 30, // regular
];

lazy_static::lazy_static! {
pub static ref ID_TO_CHAR: Vec<char> = vec![
	'i', 'j', 'l', 'o', 's', 't', 'z', ' ',
];

// block pos table
// each two lines are 4 groups, each group is a block in certaion direction
// each group has four pairs, each pair is a pos of a group
pub static ref BPT: Vec<i32> = vec![
	0, 0, 1, 0, 2, 0, 3, 0, 0, 0, 0, 1, 0, 2, 0, 3,
	0, 0, 1, 0, 2, 0, 3, 0, 0, 0, 0, 1, 0, 2, 0, 3,
	0, 0, 0, 1, 1, 1, 2, 1, 0, 0, 1, 0, 0, 1, 0, 2,
	0, 0, 1, 0, 2, 0, 2, 1, 1, 0, 1, 1, 1, 2, 0, 2,
	2, 0, 2, 1, 1, 1, 0, 1, 0, 0, 0, 1, 0, 2, 1, 2,
	0, 0, 1, 0, 2, 0, 0, 1, 0, 0, 1, 0, 1, 1, 1, 2,
	0, 0, 0, 1, 1, 0, 1, 1, 0, 0, 0, 1, 1, 0, 1, 1,
	0, 0, 0, 1, 1, 0, 1, 1, 0, 0, 0, 1, 1, 0, 1, 1,
	1, 0, 2, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 1, 2,
	1, 0, 2, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 1, 2,
	1, 0, 0, 1, 1, 1, 2, 1, 0, 0, 0, 1, 0, 2, 1, 1,
	0, 0, 1, 0, 2, 0, 1, 1, 1, 0, 1, 1, 1, 2, 0, 1,
	0, 0, 1, 0, 1, 1, 2, 1, 1, 0, 1, 1, 0, 1, 0, 2,
	0, 0, 1, 0, 1, 1, 2, 1, 1, 0, 1, 1, 0, 1, 0, 2,
];

pub static ref INITIAL_POS: Vec<i32> = vec![
	3, 3, 3, 4, 3, 3, 3
];

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

pub static ref BLOCK_HEIGHT: Vec<i32> = vec![
	1, 4, 1, 4,
	2, 3, 2, 3,
	2, 3, 2, 3,
	2, 2, 2, 2,
	2, 3, 2, 3,
	2, 3, 2, 3,
	2, 3, 2, 3,
];

pub static ref BLOCK_WIDTH: Vec<i32> = vec![
	4, 1, 4, 1,
	3, 2, 3, 2,
	3, 2, 3, 2,
	2, 2, 2, 2,
	3, 2, 3, 2,
	3, 2, 3, 2,
	3, 2, 3, 2,
];

pub static ref COLORMAP: Vec<u8> = vec![6, 4, 7, 3, 2, 5, 1, 0];

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
