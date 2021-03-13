use std::process::Command;
use std::env;
use std::path::Path;

fn main() {
	if let Ok(string) = env::var("TTTZ_MMBOT_PATH") {
		println!("cargo:rustc-cfg=feature=\"MMBot\"");
		std::fs::copy(string, env::var("OUT_DIR").unwrap() + "/mmbot").unwrap();
	}
}
