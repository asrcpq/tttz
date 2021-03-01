mod client_display;
mod client_session;
mod client_socket;
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
