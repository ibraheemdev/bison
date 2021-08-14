use crate::endpoint::Endpoint;
use crate::http::{header, Method, Request, Response, ResponseBuilder, StatusCode};
use crate::wrap::{self, Wrap};
use crate::{Body, Error, ResponseError};

use std::collections::HashMap;

use matchit::Node;

pub(crate) struct Router<W> {
    wrap: W,
    routes: HashMap<Method, Node<Box<dyn Endpoint<Request, Error = Error>>>>,
}

impl Router<wrap::Call> {
    pub(crate) fn new() -> Self {
        Self {
            wrap: wrap::Call::new(),
            routes: HashMap::with_capacity(6),
        }
    }
}

impl<W> Router<W>
where
    W: Wrap,
{
    pub(crate) fn wrap<O>(self, wrap: O) -> Router<impl Wrap>
    where
        O: Wrap,
    {
        Router {
            wrap: self.wrap.and(wrap),
            routes: self.routes,
        }
    }

    pub(crate) fn route<E, P>(
        mut self,
        method: Method,
        path: P,
        endpoint: E,
    ) -> Result<Self, matchit::InsertError>
    where
        P: Into<String>,
        E: Endpoint<Request, Error = Error> + 'static,
    {
        self.routes
            .entry(method)
            .or_default()
            .insert(path, Box::new(endpoint))?;
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
                    let endpoint = matched.value;
                    let params = matched
                        .params
                        .iter()
                        .map(|(k, v)| (k.to_owned(), v.to_owned()))
                        .collect::<Vec<_>>();
                    req.extensions_mut().insert(Params(params));
                    match endpoint.serve(req).await {
                        Ok(ok) => ok,
                        Err(err) => err.into_response_error().respond(),
                    }
                }
                Err(e) if e.tsr() && req.method() != Method::CONNECT && path != "/" => {
                    let path = if path.len() > 1 && path.ends_with('/') {
                        path[..path.len() - 1].to_owned()
                    } else {
                        format!("{}/", path)
                    };
                    ResponseBuilder::builder()
                        .header(header::LOCATION, path)
                        .status(StatusCode::PERMANENT_REDIRECT)
                        .body(Body::empty())
                        .unwrap()
                }
                Err(_) => ResponseBuilder::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Body::empty())
                    .unwrap(),
            },
            None => {
                let allowed = self.allowed_methods(path);
                if !allowed.is_empty() {
                    ResponseBuilder::builder()
                        .header(header::ALLOW, allowed.join(", "))
                        .status(StatusCode::METHOD_NOT_ALLOWED)
                        .body(Body::empty())
                        .unwrap()
                } else {
                    ResponseBuilder::builder()
                        .status(StatusCode::NOT_FOUND)
                        .body(Body::empty())
                        .unwrap()
                }
            }
        }
    }
}

// Route parameters, stored in request extensions
pub(crate) struct Params(Vec<(String, String)>);
