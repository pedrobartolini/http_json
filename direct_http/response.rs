use serde::Serialize;

#[derive(Serialize)]
pub struct Response<T> {
    status: i32,
    message: Option<String>,
    data: Option<T>,
}

impl<T> Response<T> where T: Serialize {
    pub fn ok() -> Self {
        Response {
            status: 200,
            message: None,
            data: None,
        }
    }

    pub fn internal_error() -> Self {
        Response {
            status: 500,
            message: Some("Erro interno, algo deu errado.".to_string()),
            data: None,
        }
    }

    pub fn not_found() -> Self {
        Response {
            status: 404,
            message: None,
            data: None,
        }
    }

    pub fn not_authorized() -> Self {
        Response {
            status: 403,
            message: None,
            data: None,
        }
    }

    pub fn not_logged_in() -> Self {
        Response {
            status: 401,
            message: Some("Você não está autenticado.".to_string()),
            data: None,
        }
    }

    pub fn message(self, message: String) -> Self {
        Response {
            status: self.status,
            message: Some(message),
            data: self.data,
        }
    }

    pub fn data(self, data: T) -> Self {
        Response {
            status: self.status,
            message: self.message,
            data: Some(data),
        }
    }

    pub fn finish(&self) -> Option<String> {
        match serde_json::to_string(self) {
            Ok(body) => Some(raw(200, &body)),
            Err(_) => None,
        }
    }
}

pub fn raw(status: i32, body: &str) -> String {
    format!(
        "HTTP/1.1 {}\r\nAccess-Control-Allow-Headers: Content-Type, Authorization\r\nAccess-Control-Allow-Methods: GET, POST, PUT, DELETE\r\nAccess-Control-Allow-Origin: *\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
        status,
        body.len(),
        body
    )
}
