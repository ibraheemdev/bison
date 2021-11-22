use crate::context::{Context, WithContext};
use crate::error::{IntoResponseError, ResponseError};
use crate::handler::{ErasedHandler, Handler, HandlerFn};
use crate::http::{header, Body, Method, Params, Request, Response, ResponseBuilder, StatusCode};
use crate::wrap::{And, Call, DynNext, Wrap};

use std::collections::HashMap;
use std::sync::Arc;

use matchit::Node;

pub struct Router<W> {
    wrap: W,
    routes: HashMap<Method, Node<Arc<dyn ErasedHandler>>>,
}

impl Router<Call> {
    pub(crate) fn new() -> Self {
        Self {
            wrap: Call::new(),
            routes: HashMap::with_capacity(6),
        }
    }
}

impl<W> Router<W>
where
    W: for<'r> Wrap<'r>,
{
    pub(crate) fn wrap(self, wrap: impl for<'r> Wrap<'r>) -> Router<impl for<'r> Wrap<'r>> {
        Router {
            wrap: And {
                inner: self.wrap,
                outer: wrap,
            },
            routes: self.routes,
        }
    }

    pub(crate) fn route<H, C, P>(
        mut self,
        method: Method,
        path: P,
        handler: H,
    ) -> Result<Self, matchit::InsertError>
    where
        P: Into<String>,
        H: for<'r> Handler<'r, C>,
        C: for<'r> WithContext<'r>,
    {
        let handler = HandlerFn::new({
            move |req| {
                let handler = handler.clone();
                Box::pin(async move {
                    let context = C::Context::extract(req).await?;
                    handler
                        .call(context)
                        .await
                        .map_err(IntoResponseError::into_response_error)
                })
            }
        });

        self.routes
            .entry(method)
            .or_default()
            .insert(path, Arc::new(handler))?;

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
                        .collect::<Vec<_>>();
                    req.extensions_mut().insert(Params(params));
                    match self.wrap.call(&req, DynNext::new(&*handler.clone())).await {
                        Ok(ok) => ok,
                        Err(err) => err.into_response_error().as_mut().respond(),
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
