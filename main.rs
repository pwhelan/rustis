#[allow(unused_imports)]

use std::mem::MaybeUninit;
use std::net::{SocketAddr, TcpListener};
use socket2::{Socket, Domain, Type};

fn main() {
	let srvsocket: Socket;

	let srvsocketres = Socket::new(Domain::IPV4, Type::STREAM, None);
	match srvsocketres {
		Ok(socket) => {
			srvsocket = socket
		},
		Err(error) => {
			panic!("Problem opening the socket: {}!", error);
		}
	}

	let address: SocketAddr = "0.0.0.0:35545".parse().unwrap();
	let address = address.into();
	let res = srvsocket.bind(&address);
	match res {
		Ok(_) => {},
		Err(error) => {
			panic!("unable to listen: {}!", error)
		}
	}

	let res = srvsocket.listen(5);
	match res {
		Ok(_) => {},
		Err(error) => {
			panic!("unable to listen: {}!", error)
		}
	}

	loop {
		let accepted = srvsocket.accept();
		match accepted {
			Ok((socket, _)) => {
				let mut buf: MaybeUninit<u8> = MaybeUninit::uninit();
				socket.recv(buf.as_bytes_mut());
			},
			Err(_) => {

			},
		}
	}
}