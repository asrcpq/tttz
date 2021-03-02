use std::net::UdpSocket;
use std::net::{SocketAddr, ToSocketAddrs};

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
		let mut buf = [0; 1024];
		let amt = loop {
			socket.send_to(b"new client", &target_addr).unwrap();
			std::thread::sleep(std::time::Duration::from_millis(1000));
			if let Ok(amt) = socket.recv(&mut buf) {
				if std::str::from_utf8(&buf).unwrap().starts_with("ok") {
					break amt
				}
			}
			std::thread::sleep(std::time::Duration::from_millis(1000));
		};
		let id: i32 = std::str::from_utf8(&buf[3..amt])
			.unwrap()
			.parse::<i32>()
			.unwrap();
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

impl Drop for ClientSocket {
	fn drop(&mut self) {
		self.socket.send_to(b"quit", self.addr).unwrap();
	}
}
