use smol::prelude::*;
use smol::io::{ Result, Error, ErrorKind };

use std::net::{ TcpListener, TcpStream };
use std::sync::Arc;

mod request;
mod response;
mod router;

pub use response::{ Response, ResponseBuilder, raw as RawResponse };
pub use router::{ ResponseHandler, Router };
pub use request::{ Request, Method };

#[macro_export]
macro_rules! MICRO {
	($func:expr) => {
		{
			use std::time::Instant;

			let start_time = Instant::now();
			let result = $func;
			let end_time = Instant::now();
			let elapsed_time = end_time - start_time;

			println!(
					"Time taken by {}(): {} microseconds",
					stringify!($func),
					elapsed_time.as_micros()
			);

			result
		}
	};
}

pub struct Server {
	listener: smol::Async<TcpListener>,
}

impl Server {
	pub fn new(addr: &str) -> Self {
		let addr_parsed: std::net::SocketAddr = addr.parse().expect("Failed to parse address");
		let listener = smol::Async::<TcpListener>::bind(addr_parsed).expect("Failed to create api server");

		Server {
			listener,
		}
	}

	pub fn listen(self, router: Router) {
		smol::spawn(async move { self.client_acceptor(router).await }).detach();
	}

	async fn client_acceptor(self, router: Router) {
		let router_arc = Arc::new(router);

		while let Ok((stream, _)) = self.listener.accept().await {
			let router_arc_clone = Arc::clone(&router_arc);
			smol
				::spawn(async move {
					let _ = handler(stream, router_arc_clone).await;
				})
				.detach();
		}
	}
}

pub const BUFFER_SIZE: usize = 1024;

async fn handler(mut stream: smol::Async<TcpStream>, router: Arc<Router>) -> Result<()> {
	let mut buffer = [0; BUFFER_SIZE];

	loop {
		if stream.read(&mut buffer).await? == 0 {
			break;
		}

		let mut request = match Request::new(buffer) {
			Some(request) => request,
			None => {
				break;
			}
		};

		let response = router::handle(&mut request, &router);

		stream.write(response.as_bytes()).await?;

		stream.flush().await?;

		if let Some(connection) = request.headers.get("connection") {
			if connection == "keep-alive" {
				println!("keeping alive !!");
				continue;
			} else {
				break;
			}
		}
	}

	stream.close().await
}
