use serde::Serialize;

#[derive(Serialize)]
pub struct ResponseBuilder<T> {
	status: i32,
	message: Option<String>,
	data: Option<T>,
}

impl<T> ResponseBuilder<T> where T: Serialize {
	pub fn ok() -> Self {
		ResponseBuilder {
			status: 200,
			message: None,
			data: None,
		}
	}

	pub fn internal_error() -> Self {
		ResponseBuilder {
			status: 500,
			message: Some("Erro interno, algo deu errado.".to_string()),
			data: None,
		}
	}

	pub fn not_found() -> Self {
		ResponseBuilder {
			status: 404,
			message: None,
			data: None,
		}
	}

	pub fn not_authorized() -> Self {
		ResponseBuilder {
			status: 403,
			message: None,
			data: None,
		}
	}

	pub fn not_logged_in() -> Self {
		ResponseBuilder {
			status: 401,
			message: Some("Você não está autenticado.".to_string()),
			data: None,
		}
	}

	pub fn message(self, message: String) -> Self {
		ResponseBuilder {
			status: self.status,
			message: Some(message),
			data: self.data,
		}
	}

	pub fn data(self, data: T) -> Self {
		ResponseBuilder {
			status: self.status,
			message: self.message,
			data: Some(data),
		}
	}

	pub fn finish(&self) -> Response {
		match serde_json::to_string(self) {
			Ok(response) => raw(self.status, &response),
			Err(_) => raw(500, "Internal error, faield to encode response."),
		}
	}
}

pub type Response = String;

pub fn raw(status: i32, body: &str) -> String {
	format!(
		"HTTP/1.1 {}\r\nAccess-Control-Allow-Headers: Content-Type, Authorization\r\nAccess-Control-Allow-Methods: GET, POST, PUT, DELETE\r\nAccess-Control-Allow-Origin: *\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
		status,
		body.len(),
		body
	)
}
