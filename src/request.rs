use std::collections::HashMap;

use super::{ Response, Status };

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Method {
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

#[derive(Debug)]
pub struct Request {
	pub path: String,
	pub method: Method,
	pub slugs: HashMap<String, String>,
	pub headers: HashMap<String, String>,
	pub body: Option<String>,
}

impl Request {
	pub fn new(buffer: &[u8]) -> Option<Self> {
		let mut trimmer = 0;

		let (method, method_end) = parse_method(&buffer)?;
		trimmer += method_end;

		let (path, path_end) = parse_path(&buffer[trimmer..])?;
		trimmer += path_end;

		let http_end = is_http101(&buffer[trimmer..])?;
		trimmer += http_end;

		let (headers, headers_end) = parse_headers(&buffer[trimmer..]);

		trimmer += headers_end;

		let body = {
			if headers.get("content-type") == Some(&"application/json".to_string()) {
				let content_length = headers.get("content-length")?;
				let content_length = content_length.parse::<usize>().ok()?;

				if trimmer == buffer.len() - content_length {
					Some(String::from_utf8_lossy(&buffer[trimmer..]).to_string())
				} else {
					None
				}
			} else {
				None
			}
		};

		Some(Request {
			body,
			method,
			path,
			headers,
			slugs: HashMap::new(),
		})
	}

	pub fn decode<T: serde::de::DeserializeOwned>(&self) -> Result<T, Response> {
		match &self.body {
			Some(ref body) =>
				serde_json::from_str(body).map_err(|_| Response::status(Status::BadRequest).message("Corpo da requisição inválido.")),
			None => Err(Response::status(Status::BadRequest).message("Corpo da requisição inválido.")),
		}
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

fn parse_method(buffer: &[u8]) -> Option<(Method, usize)> {
	let method_end = buffer.iter().position(|&byte| byte == b' ')?;
	let method = match &buffer[..method_end] {
		b"GET" => Method::GET,
		b"POST" => Method::POST,
		b"PUT" => Method::PUT,
		b"DELETE" => Method::DELETE,
		b"HEAD" => Method::HEAD,
		b"OPTIONS" => Method::OPTIONS,
		b"TRACE" => Method::TRACE,
		b"CONNECT" => Method::CONNECT,
		b"PATCH" => Method::PATCH,
		_ => {
			return None;
		}
	};

	Some((method, method_end + 1))
}

fn parse_path(buffer: &[u8]) -> Option<(String, usize)> {
	let path_end = buffer.iter().position(|&byte| byte == b' ')?;
	let path = String::from_utf8_lossy(&buffer[1..path_end]).to_string();

	Some((path, path_end + 1))
}

fn is_http101(buffer: &[u8]) -> Option<usize> {
	let http_end = buffer.iter().position(|&w| w == b'\r')?;

	match &buffer[..http_end] {
		b"HTTP/1.1" => Some(http_end + 2),
		_ => None,
	}
}

fn parse_headers(buffer: &[u8]) -> (HashMap<String, String>, usize) {
	let mut headers: HashMap<String, String> = HashMap::new();

	let mut trimmer = 0;

	while let Some(line_end) = buffer[trimmer..].iter().position(|&byte| byte == b'\r') {
		if let Some(separator) = buffer[trimmer..trimmer + line_end].iter().position(|&byte| byte == b':') {
			let key = String::from_utf8_lossy(&buffer[trimmer..trimmer + separator]).to_lowercase();
			let value = String::from_utf8_lossy(&buffer[trimmer + separator + 2..trimmer + line_end]).to_lowercase();
			headers.insert(key, value);
		}

		if buffer[trimmer + line_end..].starts_with("\r\n\r\n".as_bytes()) {
			trimmer += line_end + 4;
			break;
		}

		trimmer += line_end + 2;
	}

	(headers, trimmer)
}
