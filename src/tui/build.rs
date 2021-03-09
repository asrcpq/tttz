extern crate tttz_segen;
use tttz_segen::segen;
use std::env;

fn main() {
	let out_dir = env::var("OUT_DIR").unwrap();
	eprintln!("{:?}", out_dir);
	segen(std::path::PathBuf::from(out_dir));
}
