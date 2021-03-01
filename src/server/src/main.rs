mod server;
use server::Server;
mod client;
mod client_manager;

fn main() {
	let mut server = Server::new();
	server.main_loop();
}
