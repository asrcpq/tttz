use std::net::UdpSocket;
use std::net::{SocketAddr, ToSocketAddrs};
use tttz_protocol::{ClientMsg, ServerMsg};

pub struct ClientSocket {
	pub socket: UdpSocket,
	addr: SocketAddr,
}

impl ClientSocket {
	pub fn new(addr: &str) -> (ClientSocket, i32) {
		let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
		let target_addr: SocketAddr =
			addr.to_socket_addrs().unwrap().next().unwrap();
		eprintln!("{:?}", target_addr);
		socket.set_nonblocking(true).unwrap();
		let client_socket = ClientSocket {
			socket,
			addr: target_addr,
		};
		let id = loop {
			client_socket.socket.send_to(b"new_client bincode", addr).unwrap();
			std::thread::sleep(std::time::Duration::from_millis(1000));
			if let Ok(ServerMsg::AllocId(id)) = client_socket.recv() {
				break id;
			}
			std::thread::sleep(std::time::Duration::from_millis(1000));
		};
		(client_socket, id)
	}

	pub fn send(&self, buf: ClientMsg) -> std::io::Result<()> {
		self.socket.send_to(&buf.serialized(), self.addr)?;
		Ok(())
	}

	pub fn recv(&self) -> Result<ServerMsg, Box<dyn std::error::Error>> {
		let mut buf = [0; 1024];
		let amt = self.socket.recv(&mut buf)?;
		let server_msg = ServerMsg::from_serialized(&buf[..amt])?;
		Ok(server_msg)
	}
}

impl Drop for ClientSocket {
	fn drop(&mut self) {
		self.send(ClientMsg::Quit).unwrap();
	}
}
