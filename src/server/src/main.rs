mod server;
use server::Server;
mod client;
mod client_manager;

fn main() {
	let mut server = Server::new("127.0.0.1:23124");
	server.main_loop();
}
