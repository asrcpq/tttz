extern crate mypuzzle_libclient;
use mypuzzle_libclient::client_display;
use mypuzzle_libclient::client_socket;

mod client_session;
use client_session::ClientSession;

fn main() {
	let mut iter = std::env::args();
	iter.next();
	let addr = match iter.next() {
		Some(string) => string,
		None => "127.0.0.1:23124".to_string(),
	};
	let mut client_session = ClientSession::new(addr);
	client_session.main_loop()
}
