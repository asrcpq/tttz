use tttz_protocol::Display;

pub fn get_height_and_hole(display: &Display) -> ([u8; 10], i32, usize) {
	// calc height
	let mut heights = [40u8; 10];
	let mut highest_hole = 40;
	let mut highest_hole_x: i32 = -1;
	for i in 0..10 {
		let mut j: usize = 0;
		let mut state = 0;
		loop {
			if display.color[j][i] == 7 {
				if state == 1 {
					break;
				}
			} else if state == 0 {
				state = 1;
				heights[i as usize] = j as u8;
			}
			j += 1;
			if j == 40 {
				break;
			}
		}
		if j > highest_hole {
			highest_hole = j;
			highest_hole_x = i as i32;
		}
	}
	return (heights, highest_hole_x, highest_hole)
}
