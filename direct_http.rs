use std::io::{ Read, Write, Error, ErrorKind, Result };
use std::net::{ TcpListener, TcpStream };
use std::sync::Arc;

mod router;
mod request;
mod response;

pub use request::Request;
pub use response::Response;

pub struct Server {
    listener: TcpListener,
    routes: Vec<router::Route>,
}

impl Server {
    pub fn new(addr: &str) -> Self {
        let listener = TcpListener::bind(&addr).expect("Failed to create api server");

        Server {
            listener,
            routes: Vec::new(),
        }
    }

    pub fn subscribe(&mut self, pattern: &str, handler: fn(&request::Request) -> Option<String>) {
        self.routes.push(router::Route::new(pattern, handler));
    }

    pub fn subscribe_routes(&mut self, func: fn(&mut Server)) {
        func(self)
    }

    pub fn listen(self) {
        std::thread::spawn(move || self.client_acceptor());
    }

    #[tokio::main]
    async fn client_acceptor(self) {
        let routes_arc = Arc::new(self.routes);

        while let Ok((stream, _)) = self.listener.accept() {
            let routes_arc_clone = Arc::clone(&routes_arc);

            tokio::spawn(async move { handler(stream, routes_arc_clone) });
        }
    }
}

fn handler(mut stream: TcpStream, routes: Arc<Vec<router::Route>>) -> Result<()> {
    let mut buffer = [0; 1024];

    loop {
        if stream.read(&mut buffer)? == 0 {
            break;
        }

        let request_raw = String::from_utf8(buffer.to_vec())
            .map_err(|err| Error::new(ErrorKind::InvalidData, err))?
            .to_lowercase();

        let request = match request::Request::new(&request_raw) {
            Some(request) => request,
            None => {
                break;
            }
        };

        stream.write(router::response_for_request(request, &routes).as_bytes())?;

        stream.flush()?;

        if !request_raw.contains("connection: keep-alive") {
            break;
        }
    }

    stream.shutdown(std::net::Shutdown::Both)
}
