use std::collections::HashMap;

pub struct Request {
	pub path: String,
	pub method: String,
	pub slugs: HashMap<String, String>,
	pub headers: HashMap<String, String>,
}

enum Method {
	GET,
	POST,
	PUT,
	DELETE,
	HEAD,
	OPTIONS,
	TRACE,
	CONNECT,
	PATCH,
}

impl Request {
	pub fn new(mut request: &str) -> Option<Self> {
		let method_end = request.find(" ")?;
		let method_uppercase = request[..method_end].to_uppercase();
		request = &request[method_end + 1..];

		let path_end = request.find(" ")?;
		let path_lowercase = request[1..path_end].to_string();
		request = &request[path_end + 1..];

		let http_version_end = request.find("\r\n")?;
		if &request[..http_version_end].to_lowercase() != "http/1.1" {
			return None;
		}

		request = &request[http_version_end + 2..];

		let mut headers: HashMap<String, String> = HashMap::new();

		while let Some(end_split) = request.find("\r\n") {
			if let Some(middle_split) = request.find(": ") {
				headers.insert(request[..middle_split].to_lowercase(), request[middle_split + 2..end_split].to_lowercase());
			}
			request = &request[end_split + 2..];
		}

		Some(Request {
			method: method_uppercase,
			path: path_lowercase,
			headers,
			slugs: HashMap::new(),
		})
	}

	pub fn bearer_token(&self) -> Option<String> {
		match self.headers.get("authorization") {
			Some(authorization) => {
				let mut authorization = authorization.split(" ");

				if authorization.next()? != "bearer" {
					return None;
				}

				authorization.next().map(|token| token.to_string())
			}
			None => None,
		}
	}
}
