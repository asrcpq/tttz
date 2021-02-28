use std::net::UdpSocket;
use std::net::{SocketAddr, ToSocketAddrs};

pub struct ClientSocket {
	socket: UdpSocket,
	addr: SocketAddr,
}

impl ClientSocket {
	pub fn new(addr: &str) -> (ClientSocket, i32) {
		let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
		let target_addr: SocketAddr = addr.to_socket_addrs().unwrap().next().unwrap();
		eprintln!("{:?}", target_addr);
		socket.send_to(b"new client", &target_addr).unwrap();
		let mut buf = [0; 1024];
		let (amt, _) = socket.recv_from(&mut buf).unwrap();
		assert!(std::str::from_utf8(&buf).unwrap().starts_with("ok"));
		let id: i32 = std::str::from_utf8(&buf[3..amt])
			.unwrap()
			.parse::<i32>()
			.unwrap();
		socket.set_nonblocking(true).unwrap();
		(
			ClientSocket {
				socket,
				addr: target_addr,
			},
			id,
		)
	}

	pub fn send(&self, buf: &[u8]) -> std::io::Result<()> {
		self.socket.send_to(buf, self.addr)?;
		Ok(())
	}

	pub fn recv(&self, mut buf: &mut [u8]) -> std::io::Result<usize> {
		self.socket.recv(&mut buf)
	}
}
