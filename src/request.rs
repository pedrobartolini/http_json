use std::collections::HashMap;

// use super::router::Route;

pub struct Request {
	pub path: String,
	pub method: String,
	pub slugs: HashMap<String, String>,
}

impl Request {
	pub fn new(request: &str) -> Option<Self> {
		let method_end = request.find(" ")?;

		let method = request[..method_end].to_string().to_uppercase();

		let path_end = request[method_end + 1..].find(" ")?;

		let path = request[method_end + 2..path_end + method_end + 1].to_string();

		Some(Request {
			method,
			path,
			slugs: HashMap::new(),
		})
	}

	// pub fn find_route(mut self, routes: &Arc<Vec<Route>>) -> Option<String> {
	// 	for route in routes.iter() {
	// 		if self.compare_route(route) {
	// 			return (route.handler)(&self);
	// 		}
	// 	}
	// 	None
	// }

	// fn compare_route(&mut self, route: &Route) -> bool {
	// 	if route.pattern_vec_len != self.path_vec_len {
	// 		return false;
	// 	}

	// 	let mut slugs: Vec<String> = Vec::new();

	// 	for i in 0..route.pattern_vec_len {
	// 		if route.pattern_vec[i] == "{}" {
	// 			slugs.push(self.path_vec[i].to_string());
	// 			continue;
	// 		}

	// 		if route.pattern_vec[i] != self.path_vec[i] {
	// 			return false;
	// 		}
	// 	}

	// 	let slugs_len = slugs.len();

	// 	if route.required_slugs != slugs_len {
	// 		return false;
	// 	}

	// 	if slugs_len > 0 {
	// 		self.slugs = Some(slugs);
	// 	}

	// 	return true;
	// }
}
