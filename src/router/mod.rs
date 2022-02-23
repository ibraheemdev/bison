mod scope;
pub use scope::Scope;

use crate::handler::BoxHandler;
use crate::http::{ByteStr, Method, Request, Response, Status};
use crate::wrap::{And, Call, DynNext};
use crate::Wrap;

use std::collections::HashMap;

use matchit::Node;
use once_cell::sync::OnceCell;

pub struct Router<W> {
    wrap: W,
    routes: HashMap<Method, Node<BoxHandler>>,
    allowed: OnceCell<ByteStr>,
}

impl Router<Call> {
    pub(crate) fn new() -> Self {
        Self {
            wrap: Call,
            routes: HashMap::with_capacity(6),
            allowed: OnceCell::new(),
        }
    }
}

impl<W> Router<W>
where
    W: Wrap,
{
    pub(crate) fn wrap<O>(self, wrap: O) -> Router<And<W, O>>
    where
        O: Wrap,
    {
        Router {
            wrap: self.wrap.wrap(wrap),
            routes: self.routes,
            allowed: self.allowed,
        }
    }

    pub(crate) fn route(
        mut self,
        method: Method,
        path: impl Into<String>,
        handler: BoxHandler,
    ) -> Result<Self, matchit::InsertError> {
        self.routes
            .entry(method)
            .or_default()
            .insert(path, handler)?;

        Ok(self)
    }

    fn allowed(&self, path: &str) -> ByteStr {
        let mut allowed = match path {
            "*" => {
                let mut allowed = Vec::with_capacity(self.routes.len());
                for method in self
                    .routes
                    .keys()
                    .filter(|&&method| method != Method::Options)
                {
                    allowed.push(method.as_str());
                }
                allowed
            }
            _ => self
                .routes
                .keys()
                .filter(|&&method| method != Method::Options)
                .filter(|&method| {
                    self.routes
                        .get(method)
                        .map(|node| node.at(&path).is_ok())
                        .unwrap_or(false)
                })
                .map(|method| method.as_str())
                .collect(),
        };

        if !allowed.is_empty() {
            allowed.push(Method::Options.as_str())
        }

        allowed.join(", ").into()
    }

    pub(crate) async fn serve(&self, mut req: Request) -> Response {
        let path = req.uri.path();

        let node = match self.routes.get(&req.method) {
            Some(node) => node,
            None => {
                let allowed = self.allowed.get_or_init(|| self.allowed(path));

                if allowed.is_empty() {
                    return Response::from(Status::NotFound);
                }

                return Response::new()
                    .header(("Allow", allowed.clone()))
                    .status(Status::MethodNotAllowed);
            }
        };

        let matched = match node.at(path) {
            Ok(matched) => matched,
            Err(e) if e.tsr() && req.method != Method::Connect => {
                let path = if let Some(path) = path.strip_prefix('/') {
                    path.to_owned()
                } else {
                    format!("{}/", path)
                };

                return Response::new()
                    .header(("Location", path))
                    .status(Status::PermanentRedirect);
            }
            Err(_) => return Response::new().status(Status::NotFound),
        };

        let handler = matched.value;

        let params = matched
            .params
            .iter()
            .map(|(k, v)| (k.to_owned(), v.to_owned()))
            .collect::<Vec<_>>();

        req.params = params;

        match self.wrap.call(req, &DynNext(&**handler)).await {
            Ok(res) => res,
            Err(err) => err.reject(),
        }
    }
}
