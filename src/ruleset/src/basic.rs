pub const ID_TO_CHAR: [char; 8] = ['i', 'j', 'l', 'o', 's', 't', 'z', ' '];

// block pos table
// code * rotation * block * (x, y)
pub const BPT: [[[(u8, u8); 4]; 4]; 7] = [
	[
		[(0, 0), (1, 0), (2, 0), (3, 0)],
		[(0, 0), (0, 1), (0, 2), (0, 3)],
		[(0, 0), (1, 0), (2, 0), (3, 0)],
		[(0, 0), (0, 1), (0, 2), (0, 3)],
	],
	[
		[(0, 0), (1, 0), (2, 0), (0, 1)],
		[(0, 0), (0, 1), (0, 2), (1, 2)],
		[(2, 0), (2, 1), (1, 1), (0, 1)],
		[(0, 0), (1, 0), (1, 1), (1, 2)],
	],
	[
		[(0, 0), (1, 0), (2, 0), (2, 1)],
		[(0, 0), (1, 0), (0, 1), (0, 2)],
		[(0, 0), (0, 1), (1, 1), (2, 1)],
		[(1, 0), (1, 1), (1, 2), (0, 2)],
	],
	[
		[(0, 0), (0, 1), (1, 0), (1, 1)],
		[(0, 0), (0, 1), (1, 0), (1, 1)],
		[(0, 0), (0, 1), (1, 0), (1, 1)],
		[(0, 0), (0, 1), (1, 0), (1, 1)],
	],
	[
		[(0, 0), (1, 0), (1, 1), (2, 1)],
		[(1, 0), (1, 1), (0, 1), (0, 2)],
		[(0, 0), (1, 0), (1, 1), (2, 1)],
		[(1, 0), (1, 1), (0, 1), (0, 2)],
	],
	[
		[(0, 0), (1, 0), (2, 0), (1, 1)],
		[(0, 0), (0, 1), (0, 2), (1, 1)],
		[(1, 0), (0, 1), (1, 1), (2, 1)],
		[(1, 0), (1, 1), (1, 2), (0, 1)],
	],
	[
		[(1, 0), (2, 0), (0, 1), (1, 1)],
		[(0, 0), (0, 1), (1, 1), (1, 2)],
		[(1, 0), (2, 0), (0, 1), (1, 1)],
		[(0, 0), (0, 1), (1, 1), (1, 2)],
	],
];

// standard rotation pos
// each line is for a type of block, 4 pairs of pos(left up) indicates 4 directions
// each pos is the difference to first pair
pub const SRP: [[(i32, i32); 4]; 7] = [
	[(0, 0), (2, -2), (0, -1), (1, -2)],
	[(0, 0), (1, -1), (0, -1), (0, -1)],
	[(0, 0), (1, -1), (0, -1), (0, -1)],
	[(0, 0), (0, 0), (0, 0), (0, 0)],
	[(0, 0), (1, -1), (0, -1), (0, -1)],
	[(0, 0), (1, -1), (0, -1), (0, -1)],
	[(0, 0), (1, -1), (0, -1), (0, -1)],
];

pub const INITIAL_POS: [i32; 7] = [3, 3, 3, 4, 3, 3, 3];

type BlockScalar<T> = [[T; 4]; 7];

pub const BLOCK_HEIGHT: BlockScalar<i32> = [
	[1, 4, 1, 4],
	[2, 3, 2, 3],
	[2, 3, 2, 3],
	[2, 2, 2, 2],
	[2, 3, 2, 3],
	[2, 3, 2, 3],
	[2, 3, 2, 3],
];

pub const BLOCK_WIDTH: BlockScalar<i32> = [
	[4, 1, 4, 1],
	[3, 2, 3, 2],
	[3, 2, 3, 2],
	[2, 2, 2, 2],
	[3, 2, 3, 2],
	[3, 2, 3, 2],
	[3, 2, 3, 2],
];

pub const COLORMAP: [u8; 8] = [6, 4, 7, 3, 2, 5, 1, 0];

// mass center height, convenient for AI
pub const BLOCK_MCH: BlockScalar<f32> = [
	[0.0, 1.5, 0.0, 1.5],
	[0.25, 1.25, 0.75, 0.75],
	[0.25, 0.75, 0.75, 1.25],
	[0.5, 0.5, 0.5, 0.5],
	[0.5, 1.0, 0.5, 1.0],
	[0.25, 1.0, 0.75, 1.0],
	[0.5, 1.0, 0.5, 1.0],
];

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn test_block_height_width() {
		let mut height: BlockScalar<i32> = Default::default();
		let mut width: BlockScalar<i32> = Default::default();
		for code in 0..7 {
			for rot in 0..4 {
				height[code][rot] = BPT[code][rot].iter()
					.fold(0, |max, data| if data.1 > max {data.1} else {max}) as i32
					+ 1;
				width[code][rot] = BPT[code][rot].iter()
					.fold(0, |max, data| if data.0 > max {data.0} else {max}) as i32
					+ 1;
			}
		}
		assert_eq!(height, BLOCK_HEIGHT);
		assert_eq!(width, BLOCK_WIDTH);
	}

	#[test]
	fn test_mch() {
		let mut mch: BlockScalar<f32> = Default::default();
		for code in 0..7 {
			for rot in 0..4 {
				mch[code][rot] = BPT[code][rot].iter()
					.fold(0f32, |max, data| max + data.1 as f32)
					/ 4.0;
			}
		}
		assert_eq!(mch, BLOCK_MCH);
	}
}
