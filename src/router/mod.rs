mod scope;
pub use scope::Scope;

use crate::http::{self, Body, Method, Request, Response, ResponseBuilder, StatusCode};
use crate::reject::IntoRejection;
use crate::state::AppState;
use crate::wrap::{Call, Wrap};
use crate::{handler, Context, Respond};
use ::http::header; // TODO

use std::collections::HashMap;

use ::http::Method as HttpMethod;
use matchit::Node;

pub struct Router<W> {
    wrap: W,
    routes: HashMap<HttpMethod, Node<handler::Erased>>,
}

impl Router<Call> {
    pub(crate) fn new() -> Self {
        Self {
            wrap: Call::new(),
            routes: HashMap::with_capacity(6),
        }
    }
}

impl<'g, W> Router<W>
where
    W: Wrap<'g, &'g Request>,
{
    pub(crate) fn wrap<O, C>(self, wrap: O) -> Router<impl Wrap<'g, &'g Request>>
    where
        O: Wrap<'g, C>,
        C: Context<'g>,
    {
        Router {
            wrap: self.wrap.wrap(wrap),
            routes: self.routes,
        }
    }

    pub(crate) fn route(
        mut self,
        method: Method,
        path: impl Into<String>,
        handler: handler::Erased,
    ) -> Result<Self, matchit::InsertError> {
        self.routes
            .entry(method.into_http())
            .or_default()
            .insert(path, handler)?;

        Ok(self)
    }

    fn allowed_methods(&self, path: &str) -> Vec<&str> {
        let mut allowed = match path {
            "*" => {
                let mut allowed = Vec::with_capacity(self.routes.len());
                for method in self
                    .routes
                    .keys()
                    .filter(|&method| method != HttpMethod::OPTIONS)
                {
                    allowed.push(method.as_ref());
                }
                allowed
            }
            _ => self
                .routes
                .keys()
                .filter(|&method| method != HttpMethod::OPTIONS)
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
            allowed.push(HttpMethod::OPTIONS.as_str())
        }

        allowed
    }

    pub(crate) async fn serve(&self, req: ::http::Request<Body>, state: AppState) -> Response {
        let path = req.uri().path();
        match self.routes.get(req.method()) {
            Some(node) => match node.at(path) {
                Ok(matched) => {
                    let handler = matched.value;

                    let params = matched
                        .params
                        .iter()
                        .map(|(k, v)| (k.to_owned(), v.to_owned()))
                        .collect::<http::request::Params>();

                    let req = match Request::new(req, state, params) {
                        Some(req) => req,
                        None => {
                            return ResponseBuilder::new()
                                .status(StatusCode::METHOD_NOT_ALLOWED)
                                .body(Body::empty())
                                .unwrap()
                        }
                    };

                    match self.wrap.call(&req, handler).await {
                        Ok(ok) => match ok.respond() {
                            Ok(ok) => ok,
                            Err(err) => err.into_response_error().reject(&req),
                        },
                        Err(err) => err.into_response_error().reject(&req),
                    }
                }
                Err(e) if e.tsr() && req.method() != HttpMethod::CONNECT && path != "/" => {
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
