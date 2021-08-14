use std::convert::Infallible;
use std::future::{ready, Future, Ready};
use std::pin::Pin;
use std::sync::Arc;
use std::task::Context;
use std::task::Poll;

use bison::Bison;
use futures_core::Stream;
use hyper::service::Service;

pub use hyper::Server;

pub struct BisonService<W> {
    bison: Arc<Bison<W>>,
}

impl<W> Clone for BisonService<W> {
    fn clone(&self) -> Self {
        Self {
            bison: self.bison.clone(),
        }
    }
}

pub struct BisonHttpBody {
    inner: bison::Body,
}

impl http_body::Body for BisonHttpBody {
    type Data = bison::Bytes;
    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn poll_data(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Data, Self::Error>>> {
        Pin::new(&mut self.inner).poll_next(cx)
    }

    fn poll_trailers(
        self: Pin<&mut Self>,
        _: &mut Context<'_>,
    ) -> Poll<Result<Option<hyper::HeaderMap>, Self::Error>> {
        Poll::Ready(Ok(None))
    }

    /// will equal the lower bound.
    fn size_hint(&self) -> http_body::SizeHint {
        // TODO: SizeHint::default()
        todo!()
    }
}

impl<W> Service<hyper::Request<hyper::Body>> for BisonService<W>
where
    W: bison::Wrap + Send + Sync,
{
    type Response = hyper::Response<BisonHttpBody>;
    type Error = Infallible;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: hyper::Request<hyper::Body>) -> Self::Future {
        let (parts, body) = req.into_parts();
        let req = hyper::Request::from_parts(parts, bison::Body::stream(body));
        let bison = self.bison.clone();

        Box::pin(async move {
            let resp = bison.serve(req).await;
            let (parts, body) = resp.into_parts();
            let resp = hyper::Response::from_parts(parts, BisonHttpBody { inner: body });
            Ok(resp)
        })
    }
}

pub struct BisonMakeService<W> {
    service: BisonService<W>,
}

impl<W> BisonMakeService<W> {
    pub fn new(bison: Bison<W>) -> Self {
        Self {
            service: BisonService {
                bison: Arc::new(bison),
            },
        }
    }
}

impl<T, W> Service<T> for BisonMakeService<W> {
    type Response = BisonService<W>;
    type Error = Infallible;
    type Future = Ready<Result<Self::Response, Infallible>>;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _: T) -> Self::Future {
        ready(Ok(self.service.clone()))
    }
}
