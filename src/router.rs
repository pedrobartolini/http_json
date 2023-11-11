use std::collections::HashMap;
use std::sync::Arc;

use super::request::Request;
use super::response::{ raw, Response };

pub struct Router {
	root: Node,
}
#[derive(Debug)]
struct Node {
	handler: Option<fn(Request) -> Response>,
	children: HashMap<&'static str, Node>,
	slug_placeholder: Option<String>,
}

impl Router {
	pub fn new() -> Self {
		Router {
			root: Node {
				handler: None,
				slug_placeholder: None,
				children: HashMap::new(),
			},
		}
	}

	pub fn subscribe_route(&mut self, route: &'static str, handler: fn(Request) -> Response) {
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
				handler: None,
				slug_placeholder,
				children: HashMap::new(),
			});
		}

		current.handler = Some(handler);
	}
}

pub fn handle(mut request: Request, router: &Arc<Router>) -> String {
	let mut current = &router.root;

	for slug in request.path.split("/") {
		match current.children.get(slug) {
			Some(node) => {
				current = node;
			}
			None =>
				match current.children.get("{}") {
					Some(node) => {
						let slug_placeholder = node.slug_placeholder.clone();
						request.slugs.insert(slug_placeholder.unwrap(), slug.to_string());
						current = node;
					}
					None => {
						return raw(404, "Not Found");
					}
				}
		}
	}

	println!("final node: {:?}", current);

	match current.handler {
		Some(handler) => {
			match handler(request) {
				Some(response) => response,
				None => raw(500, "Internal Server Error"),
			}
		}
		None => raw(404, "Not Found"),
	}
}
