use serde_json::{ json, Value };

#[macro_export]
macro_rules! ENCODE {
	($data:expr) => {
		serde_json::to_value($data)
	};
}

pub struct Response {
	status: i32,
	message: Option<String>,
	data: Option<Value>,
}

impl Response {
	pub fn status(status: i32) -> Self {
		Response {
			status,
			message: None,
			data: None,
		}
	}

	pub fn message(self, message: &str) -> Self {
		Response {
			status: self.status,
			message: Some(message.to_string()),
			data: None,
		}
	}

	pub fn data(self, encoded: Result<Value, serde_json::Error>) -> Response {
		match encoded {
			Ok(data) =>
				Response {
					status: self.status,
					message: self.message,
					data: Some(data),
				},
			Err(_) =>
				Response {
					status: 500,
					message: Some("Erro interno. Falha ao codificar resposta.".to_string()),
					data: None,
				},
		}
	}

	pub fn finish(self) -> String {
		let json = json!({
			"status": self.status,
			"message": self.message,
			"data": self.data,
		}).to_string();

		format!(
			"HTTP/1.1 200\r\nAccess-Control-Allow-Headers: Content-Type, Authorization\r\nAccess-Control-Allow-Methods: GET, POST, PUT, DELETE\r\nAccess-Control-Allow-Origin: *\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
			json.len(),
			json
		)
	}
}
