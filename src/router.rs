//mod scope;
//pub use scope::Scope;

use crate::handler;
use crate::http::{self, header, Body, Method, Request, Response, ResponseBuilder, StatusCode};
use crate::reject::IntoRejection;
use crate::respond::Respond;
use crate::wrap::{Call, Wrap};

use std::collections::HashMap;

use matchit::Node;

pub struct Router<W> {
    wrap: W,
    routes: HashMap<Method, Node<Box<handler::Erased>>>,
}

impl Router<Call> {
    pub(crate) fn new() -> Self {
        Self {
            wrap: Call::new(),
            routes: HashMap::with_capacity(6),
        }
    }
}

trait AnyWrap: for<'req> Wrap<'req, &'req mut Request> {}
impl<W> AnyWrap for W where W: for<'req> Wrap<'req, &'req mut Request> {}

impl<W> Router<W>
where
    W: AnyWrap,
{
    pub(crate) fn wrap(self, wrap: impl Wrap) -> Router<impl Wrap> {
        Router {
            wrap: self.wrap.and(wrap),
            routes: self.routes,
        }
    }

    pub(crate) fn route(
        mut self,
        method: Method,
        path: impl Into<String>,
        handler: Box<handler::Erased>,
    ) -> Result<Self, matchit::InsertError> {
        self.routes.entry(method).or_default().insert(
            path,
            Box::new(handler::BoxReturn::new(handler::Extract::new(handler))),
        )?;

        Ok(self)
    }

    fn allowed_methods(&self, path: &str) -> Vec<&str> {
        let mut allowed = match path {
            "*" => {
                let mut allowed = Vec::with_capacity(self.routes.len());
                for method in self
                    .routes
                    .keys()
                    .filter(|&method| method != Method::OPTIONS)
                {
                    allowed.push(method.as_ref());
                }
                allowed
            }
            _ => self
                .routes
                .keys()
                .filter(|&method| method != Method::OPTIONS)
                .filter(|&method| {
                    self.routes
                        .get(method)
                        .map(|node| node.at(&path).is_ok())
                        .unwrap_or(false)
                })
                .map(AsRef::as_ref)
                .collect(),
        };

        if !allowed.is_empty() {
            allowed.push(Method::OPTIONS.as_str())
        }

        allowed
    }

    pub(crate) async fn serve(&self, mut req: Request) -> Response {
        let path = req.uri().path();
        match self.routes.get(req.method()) {
            Some(node) => match node.at(path) {
                Ok(matched) => {
                    let handler = matched.value;

                    let params = matched
                        .params
                        .iter()
                        .map(|(k, v)| (k.to_owned(), v.to_owned()))
                        .collect::<http::ext::Params>();

                    req.extensions_mut().insert(params);

                    match self.wrap.call(&req, &DynNext::new(&**handler)).await {
                        Ok(ok) => match ok.respond() {
                            Ok(ok) => ok,
                            Err(err) => err.into_response_error().reject(&req),
                        },
                        Err(err) => err.into_response_error().reject(&req),
                    }
                }
                Err(e) if e.tsr() && req.method() != Method::CONNECT && path != "/" => {
                    let path = if path.len() > 1 && path.ends_with('/') {
                        path[..path.len() - 1].to_owned()
                    } else {
                        format!("{}/", path)
                    };
                    ResponseBuilder::new()
                        .header(header::LOCATION, path)
                        .status(StatusCode::PERMANENT_REDIRECT)
                        .body(Body::empty())
                        .unwrap()
                }
                Err(_) => ResponseBuilder::new()
                    .status(StatusCode::NOT_FOUND)
                    .body(Body::empty())
                    .unwrap(),
            },
            None => {
                let allowed = self.allowed_methods(path);
                if !allowed.is_empty() {
                    ResponseBuilder::new()
                        .header(header::ALLOW, allowed.join(", "))
                        .status(StatusCode::METHOD_NOT_ALLOWED)
                        .body(Body::empty())
                        .unwrap()
                } else {
                    ResponseBuilder::new()
                        .status(StatusCode::NOT_FOUND)
                        .body(Body::empty())
                        .unwrap()
                }
            }
        }
    }
}
