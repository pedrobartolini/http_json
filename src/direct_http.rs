use std::io::{ Read, Write, Error, ErrorKind, Result };
use std::net::{ TcpListener, TcpStream };
use std::sync::Arc;

mod router;
mod request;
mod response;

pub use request::Request;
pub use response::{ ResponseBuilder, Response };
pub use router::{ Router };

pub struct Server {
	listener: TcpListener,
}

impl Server {
	pub fn new(addr: &str) -> Self {
		let listener = TcpListener::bind(&addr).expect("Failed to create api server");

		Server {
			listener,
		}
	}

	pub fn listen(self, router: Router) {
		std::thread::spawn(move || self.client_acceptor(router));
	}

	#[tokio::main]
	async fn client_acceptor(self, router: Router) {
		let router_arc = Arc::new(router);

		while let Ok((stream, _)) = self.listener.accept() {
			let router_arc_clone = Arc::clone(&router_arc);
			tokio::spawn(async move { handler(stream, router_arc_clone) });
		}
	}
}

fn handler(mut stream: TcpStream, router: Arc<Router>) -> Result<()> {
	let mut buffer = [0; 1024];

	loop {
		if stream.read(&mut buffer)? == 0 {
			break;
		}

		let request_raw = String::from_utf8(buffer.to_vec())
			.map_err(|err| Error::new(ErrorKind::InvalidData, err))?
			.to_lowercase();

		let request = match request::Request::new(&request_raw) {
			Some(request) => request,
			None => {
				break;
			}
		};

		stream.write(router::handle(request, &router).as_bytes())?;

		stream.flush()?;

		if !request_raw.contains("connection: keep-alive") {
			break;
		}
	}

	stream.shutdown(std::net::Shutdown::Both)
}
