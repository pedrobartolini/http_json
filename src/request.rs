use std::sync::Arc;

use super::router::Route;

pub struct Request {
	path_vec: Vec<String>,
	path_vec_len: usize,
	pub method: String,
	pub slugs: Option<Vec<String>>,
}

impl Request {
	pub fn new(request: &str) -> Option<Self> {
		let method_end = request.find(" ")? + 1;

		let method = request[..method_end].to_string();

		let path_end = request[method_end..].find(" ")?;

		let path_vec: Vec<String> = request[method_end..method_end + path_end]
			.split("/")
			.map(|e| e.to_string())
			.collect();

		let path_vec_len = path_vec.len();

		Some(Request {
			method,
			path_vec,
			path_vec_len,
			slugs: None,
		})
	}

	pub fn find_route(mut self, routes: &Arc<Vec<Route>>) -> Option<String> {
		for route in routes.iter() {
			if self.compare_route(route) {
				return (route.handler)(&self);
			}
		}
		None
	}

	fn compare_route(&mut self, route: &Route) -> bool {
		if route.pattern_vec_len != self.path_vec_len {
			return false;
		}

		let mut slugs: Vec<String> = Vec::new();

		for i in 0..route.pattern_vec_len {
			if route.pattern_vec[i] == "{}" {
				slugs.push(self.path_vec[i].to_string());
				continue;
			}

			if route.pattern_vec[i] != self.path_vec[i] {
				return false;
			}
		}

		if slugs.len() > 0 {
			self.slugs = Some(slugs);
		}

		return true;
	}
}
