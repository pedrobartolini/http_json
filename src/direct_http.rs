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
		smol::spawn(async move { self.client_acceptor(router, limiter).await }).detach();
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

const BUFFER_SIZE: usize = 2048;
const TIMEOUT: u64 = 60;

async fn handler(mut stream: smol::Async<TcpStream>, addr: SocketAddr, router: &'static Router, limiter: Limiter) -> Result<()> {
	let mut buffer = [0; BUFFER_SIZE];

	let ip = addr.ip().to_string();

	loop {
		let mut keep_alive = false;

		if stream.read_timeout().await {
			break;
		}

		let bytes_read = stream.read(&mut buffer).await?;

		if bytes_read == 0 {
			break;
		}

		let response = {
			if !limiter.server_allow() || !limiter.client_allow(ip.clone()) {
				Response::status(429).message("Muitas requisições foram feitas. Tente novamente em alguns segundos.").finish()
			} else {
				match Request::new(&buffer[..bytes_read]) {
					Some(mut request) => {
						keep_alive = request.headers.get("connection").is_some_and(|value| value == "keep-alive");
						router::handle(&mut request, &router).finish()
					}
					None => {
						continue;
					}
				}
			}
		};

		if stream.write_timeout().await {
			break;
		}

		stream.write(response.as_bytes()).await?;

		stream.flush().await?;

		if !keep_alive {
			break;
		}
	}

	stream.close().await
}

#[async_trait::async_trait]
trait Timeout {
	async fn write_timeout(&self) -> bool;
	async fn read_timeout(&self) -> bool;
}

#[async_trait::async_trait]
impl Timeout for smol::Async<TcpStream> {
	async fn write_timeout(&self) -> bool {
		smol::future::race(async { Ok(self.writable()) }, async { Err(timer(TIMEOUT).await) }).await.is_err()
	}

	async fn read_timeout(&self) -> bool {
		smol::future::race(async { Ok(self.readable()) }, async { Err(timer(TIMEOUT).await) }).await.is_err()
	}
}

async fn timer(seconds: u64) {
	smol::Timer::after(std::time::Duration::from_secs(seconds)).await;
}
