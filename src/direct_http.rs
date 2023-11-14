use smol::prelude::*;
use smol::io::Result;

use std::net::{ TcpListener, TcpStream, SocketAddr };

mod request;
mod response;
mod router;
mod rate_limiter;

pub use response::Response;
pub use router::{ ResponseHandler, Router };
pub use request::{ Request, Method };
pub use rate_limiter::MutexLimiter as Limiter;

#[macro_export]
macro_rules! MICRO {
	($func:expr) => {
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

	pub fn listen(self, router: Router, limiter: Limiter) {
		limiter.start();

		let router: &'static Router = Box::leak(Box::new(router));

		smol::spawn(async move { self.client_acceptor(&router, limiter).await }).detach();
	}

	async fn client_acceptor(self, router: &'static Router, limiter: Limiter) {
		while let Ok((stream, addr)) = self.listener.accept().await {
			let limiter_clone = limiter.clone();
			smol
				::spawn(async move {
					let _ = handler(stream, addr, router, limiter_clone).await;
				})
				.detach();
		}
	}
}

const BUFFER_SIZE: usize = 1024;

async fn handler(mut stream: smol::Async<TcpStream>, addr: SocketAddr, router: &'static Router, limiter: Limiter) -> Result<()> {
	let mut buffer = [0; BUFFER_SIZE];

	// TODO INSERT READ TIMEOUT

	let ip: &'static str = Box::leak(addr.ip().to_string().into_boxed_str());

	loop {
		if stream.read(&mut buffer).await? == 0 {
			break;
		}

		if !limiter.server_allow() || !limiter.client_allow(&ip) {
			stream.write(Response::status(429).message("Muitas requisições foram feitas. Tente novamente em alguns segundos.").finish().as_bytes()).await?;

			stream.flush().await?;
		} else {
			let mut request = match Request::new(buffer) {
				Some(request) => request,
				None => {
					continue;
				}
			};

			let response = router::handle(&mut request, &router);

			stream.write(response.finish().as_bytes()).await?;

			stream.flush().await?;

			if let Some(connection) = request.headers.get("connection") {
				if connection == "keep-alive" {
					continue;
				} else {
					break;
				}
			}
		}
	}

	stream.close().await
}
