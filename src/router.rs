use std::collections::HashMap;

use super::request::{ Request, Method };
use super::response::Response;

pub type ResponseHandler = Box<dyn (Fn(&Request) -> Response) + Send + Sync>;

struct Node {
	route: Option<Route>,
	slug_placeholder: Option<String>,
	children: HashMap<&'static str, Node>,
}

struct Route {
	pub handlers: HashMap<Method, ResponseHandler>,
}

pub struct Router {
	root: Node,
}

impl Router {
	pub fn new() -> Self {
		Router {
			root: Node {
				slug_placeholder: None,
				route: None,
				children: HashMap::new(),
			},
		}
	}

	pub fn subscribe_route(&mut self, method: Method, route: &'static str, handler: ResponseHandler) {
		let mut current = &mut self.root;

		for (i, mut slug) in route.split("/").enumerate() {
			if i == 0 && slug == "" {
				continue;
			}

			let mut slug_placeholder: Option<String> = None;

			if slug.starts_with("{") && slug.ends_with("}") {
				slug_placeholder = Some(slug[1..slug.len() - 1].to_string());
				slug = "{}";
			}

			current = current.children.entry(slug).or_insert_with(|| Node {
				route: None,
				slug_placeholder,
				children: HashMap::new(),
			});
		}

		match current.route {
			Some(ref mut route) => {
				route.handlers.insert(method, handler);
			}
			None => {
				let mut handlers = HashMap::new();
				handlers.insert(method, handler);
				current.route = Some(Route {
					handlers,
				});
			}
		}
	}
}

pub fn handle(request: &mut Request, router: &'static Router) -> Response {
	let mut current = &router.root;

	for slug in request.path.split("/") {
		match current.children.get(slug) {
			Some(node) => {
				current = node;
			}
			None =>
				match current.children.get("{}") {
					Some(node) => {
						let slug_placeholder = node.slug_placeholder.clone().unwrap();
						request.slugs.insert(slug_placeholder, slug.to_string());
						current = node;
					}
					None => {
						return Response::status(404).message("Não encontrado.");
					}
				}
		}
	}

	match &current.route {
		Some(route) => {
			match route.handlers.get(&request.method) {
				Some(handler) => handler(request),
				None => Response::status(405).message(format!("Métodos disponíveis : {:?}", route.handlers.keys()).as_str()),
			}
		}
		None => Response::status(404).message("Não encontrado."),
	}
}
