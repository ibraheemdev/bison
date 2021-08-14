use std::cell::RefCell;
use std::future::{ready, Future, Ready};
use std::pin::Pin;
use std::rc::Rc;
use std::task::Context;
use std::task::Poll;

use actix_service::{Service, ServiceFactory};
use bison::Bison;
use futures_core::Stream;

pub struct BisonService<W> {
    bison: Rc<Bison<W>>,
}

pub fn to_bison_request(actix_req: actix_http::Request) -> bison::Request {
    let (mut actix_head, actix_payload) = actix_req.into_parts();
    let actix_head = std::mem::take(&mut *actix_head);
    let mut bison_req = bison::http::request::Builder::new()
        .uri(actix_head.uri)
        .method(actix_head.method)
        .version(actix_head.version)
        .body(bison::Body::stream(actix_payload))
        .unwrap();
    *bison_req.headers_mut() = actix_head.headers.into_iter().collect();
    bison_req
}

pub fn to_actix_response(bison_resp: bison::Response) -> actix_http::Response<BisonMessageBody> {
    let (bison_parts, bison_body) = bison_resp.into_parts();
    let mut actix_resp = actix_http::Response::build(bison_parts.status)
        .message_body(BisonMessageBody { inner: bison_body })
        .unwrap();
    actix_resp.head_mut().version = bison_parts.version;
    *actix_resp.headers_mut() = bison_parts.headers.into();
    actix_resp
}

impl<W> Service<actix_http::Request> for BisonService<W>
where
    W: bison::Wrap + 'static,
{
    type Response = actix_http::Response<BisonMessageBody>;
    type Error = actix_http::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&self, req: actix_http::Request) -> Self::Future {
        let bison = self.bison.clone();

        Box::pin(async move {
            let resp = bison.serve(to_bison_request(req)).await;
            Ok(to_actix_response(resp))
        })
    }
}

pub struct BisonServiceFactory<W> {
    service: RefCell<Option<BisonService<W>>>,
}

impl<W> BisonServiceFactory<W> {
    pub fn new(bison: Bison<W>) -> Self {
        Self {
            service: RefCell::new(Some(BisonService {
                bison: Rc::new(bison),
            })),
        }
    }
}

pub struct BisonMessageBody {
    inner: bison::Body,
}

impl actix_http::body::MessageBody for BisonMessageBody {
    type Error = Box<dyn std::error::Error>;

    fn size(&self) -> actix_http::body::BodySize {
        match &self.inner {
            bison::Body::Stream(_) => actix_http::body::BodySize::Stream,
            bison::Body::Once(bytes) => actix_http::body::BodySize::Sized(bytes.len() as _),
            bison::Body::Empty => actix_http::body::BodySize::Empty,
            _ => unimplemented!(),
        }
    }

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<bison::Bytes, Self::Error>>> {
        Pin::new(&mut self.inner).poll_next(cx)
    }
}

impl<W> ServiceFactory<actix_http::Request> for BisonServiceFactory<W>
where
    W: bison::Wrap + 'static,
{
    type Response = actix_http::Response<BisonMessageBody>;
    type Error = actix_http::Error;
    type Config = ();
    type Service = BisonService<W>;
    type InitError = ();
    type Future = Ready<Result<Self::Service, ()>>;

    fn new_service(&self, _: Self::Config) -> Self::Future {
        ready(Ok(self.service.borrow_mut().take().unwrap()))
    }
}
