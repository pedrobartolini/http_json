use std::collections::HashMap;
use std::sync::{ Mutex, Arc };

pub struct MutexLimiter(Arc<Mutex<Limiter>>);

impl MutexLimiter {
	pub fn new(limit_per_client: i32, limit_per_server: i32) -> Self {
		MutexLimiter(Arc::new(Mutex::new(Limiter::new(limit_per_client, limit_per_server))))
	}

	pub fn start(&self) {
		let clone = self.clone();
		std::thread::spawn(move || {
			loop {
				clone.decrease_all();
				std::thread::sleep(std::time::Duration::from_millis(100));
			}
		});

		let clone = self.clone();
		std::thread::spawn(move || {
			loop {
				clone.remove_empty_clients();
				std::thread::sleep(std::time::Duration::from_millis(120000));
			}
		});
	}

	fn decrease_all(&self) {
		let mut limiter = self.0.lock().unwrap();
		limiter.decrease_all();
	}

	fn remove_empty_clients(&self) {
		let mut limiter = self.0.lock().unwrap();
		limiter.remove_empty_clients();
	}

	pub fn clone(&self) -> Self {
		MutexLimiter(Arc::clone(&self.0))
	}

	pub fn server_allow(&self) -> bool {
		let mut limiter = self.0.lock().unwrap();
		limiter.server_allow()
	}

	pub fn client_allow(&self, ip: String) -> bool {
		let mut limiter = self.0.lock().unwrap();
		limiter.client_allow(ip)
	}
}

pub struct Limiter {
	pub limit_per_client: i32,
	pub limit_per_server: i32,

	client_decreaser: i32,
	server_decreaser: i32,

	server: i32,

	clients: HashMap<String, i32>,
}

impl Limiter {
	pub fn new(mut limit_per_client: i32, mut limit_per_server: i32) -> Self {
		if limit_per_client < 10 {
			limit_per_client = 10;
		}

		if limit_per_server < 10 {
			limit_per_server = 10;
		}

		Limiter {
			limit_per_client,
			limit_per_server,

			client_decreaser: limit_per_client / 10,
			server_decreaser: limit_per_server / 10,

			server: 0,
			clients: HashMap::new(),
		}
	}

	fn remove_empty_clients(&mut self) {
		self.clients.retain(|_, v| *v > 0);
	}

	fn decrease_all(&mut self) {
		if self.server > 0 {
			self.server -= self.server_decreaser;
		}

		for client in self.clients.values_mut() {
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

	fn client_allow(&mut self, ip: String) -> bool {
		let client = self.clients.entry(ip).or_insert(0);

		*client += 1;

		if *client > self.limit_per_client {
			*client -= 1;
			false
		} else {
			true
		}
	}
}
