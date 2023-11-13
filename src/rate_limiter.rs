use std::collections::HashMap;
use std::sync::{ Mutex, Arc };
use std::net::SocketAddr;

pub struct MutexLimiter(Arc<Mutex<Limiter>>);

impl MutexLimiter {
	pub fn new(limit_per_client: i32, limit_per_server: i32) -> Self {
		MutexLimiter(Arc::new(Mutex::new(Limiter::new(limit_per_client, limit_per_server))))
	}

	pub fn start(&self) {
		let clone = self.clone();

		std::thread::spawn(move || {
			clone.decrease_all();
			std::thread::sleep(std::time::Duration::from_millis(100));
		});
	}

	fn decrease_all(&self) {
		let mut limiter = self.0.lock().unwrap();
		limiter.decrease_all();
	}

	pub fn clone(&self) -> Self {
		MutexLimiter(Arc::clone(&self.0))
	}

	pub fn server_allow(&self) -> bool {
		let mut limiter = self.0.lock().unwrap();
		limiter.server_allow()
	}

	pub fn client_allow(&self, addr: SocketAddr) -> bool {
		let mut limiter = self.0.lock().unwrap();
		limiter.client_allow(addr)
	}
}

struct Limiter {
	pub limit_per_client: i32,
	pub limit_per_server: i32,

	client_decreaser: i32,
	server_decreaser: i32,

	server: i32,

	clients: HashMap<SocketAddr, i32>,
}

impl Limiter {
	pub fn new(limit_per_client: i32, limit_per_server: i32) -> Self {
		Limiter {
			limit_per_client,
			limit_per_server,

			client_decreaser: limit_per_client / 10,
			server_decreaser: limit_per_server / 10,

			server: 0,
			clients: HashMap::new(),
		}
	}

	fn decrease_all(&mut self) {
		if self.server > 0 {
			self.server -= self.server_decreaser;
		}

		for (_, client) in self.clients.iter_mut() {
			if *client > 0 {
				*client -= self.client_decreaser;
			}
		}
	}

	fn server_allow(&mut self) -> bool {
		self.server += 1;

		if self.server > self.limit_per_server {
			self.server -= 1;
			false
		} else {
			true
		}
	}

	fn client_allow(&mut self, addr: SocketAddr) -> bool {
		let client = self.clients.entry(addr).or_insert(0);

		*client += 1;

		if *client > self.limit_per_client {
			*client -= 1;
			false
		} else {
			true
		}
	}
}
