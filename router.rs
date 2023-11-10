use std::sync::Arc;
use super::request::Request;
use super::response::raw;

pub struct Route {
	pub pattern_vec: Vec<String>,
	pub pattern_vec_len: usize,
	pub handler: fn(&Request) -> Option<String>,
}

impl Route {
	pub fn new(pattern: &str, handler: fn(&Request) -> Option<String>) -> Self {
		let pattern_vec = pattern
			.split("/")
			.map(|e| e.to_string())
			.collect::<Vec<String>>();
		let pattern_vec_len = pattern_vec.len();

		Route {
			pattern_vec,
			pattern_vec_len,
			handler,
		}
	}
}

pub fn response_for_request(request: Request, routes: &Arc<Vec<Route>>) -> String {
	match request.find_route(routes) {
		Some(response) => response,
		None => raw(404, "not found"),
	}
}
