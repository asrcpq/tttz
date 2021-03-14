use tttz_libclient::client_socket;

mod client_session;
use client_session::ClientSession;
mod client_display;
mod sound_manager;
mod keymap;

fn main() {
	let mut iter = std::env::args();
	iter.next();
	let mut addr = "127.0.0.1:23124".to_string();
	let mut cmds: Vec<String> = Vec::new();
	while let Some(cmd) = iter.next() {
		if cmd == "addr" {
			if let Some(string) = iter.next() {
				addr = string;
			}
		}
		if cmd == "execute" {
			if let Some(string) = iter.next() {
				cmds.push(string);
			}
		}
	}
	let mut client_session = ClientSession::new(addr);
	for line in cmds.drain(..) {
		println!("executing {}", line);
		client_session.proc_line(&line);
	}
	client_session.main_loop()
}
