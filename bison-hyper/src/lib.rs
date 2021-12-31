use std::convert::Infallible;
use std::future::{ready, Future, Ready};
use std::io;
use std::net::*;
use std::pin::Pin;
use std::sync::Arc;
use std::task::Context;
use std::task::Poll;

use bison::Bison;
use futures_core::Stream;
use hyper::server::conn::AddrIncoming;
use hyper::service::Service;

pub use hyper::Server;

pub trait Serve<W> {
    fn serve(self, addr: impl ToSocketAddr) -> Server<AddrIncoming, BisonMakeService<W>>;
    fn into_make_service(self) -> BisonMakeService<W>;
    fn into_service(self) -> BisonService<W>;
}

impl<W> Serve<W> for Bison<W>
where
    W: bison::Wrap + 'static,
{
    fn serve(self, addr: impl ToSocketAddr) -> Server<AddrIncoming, BisonMakeService<W>> {
        let addr = addr.to_socket_addr().expect("failed to create socket addr");
        hyper::Server::bind(&addr).serve(self.into_make_service())
    }

    fn into_make_service(self) -> BisonMakeService<W> {
        BisonMakeService {
            service: self.into_service(),
        }
    }

    fn into_service(self) -> BisonService<W> {
        BisonService {
            bison: Arc::new(self),
        }
    }
}

pub struct BisonMakeService<W> {
    service: BisonService<W>,
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

pub struct BisonService<W> {
    bison: Arc<Bison<W>>,
}

impl<W> Service<hyper::Request<hyper::Body>> for BisonService<W>
where
    W: bison::Wrap + 'static,
{
    type Response = hyper::Response<BisonHttpBody>;
    type Error = Infallible;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: hyper::Request<hyper::Body>) -> Self::Future {
        let (parts, body) = req.into_parts();
        let req = hyper::Request::from_parts(parts, bison::http::Body::stream(body));
        let bison = self.bison.clone();

        Box::pin(async move {
            let resp = bison.serve_one(req).await;
            let (parts, body) = resp.into_parts();
            let resp = hyper::Response::from_parts(parts, BisonHttpBody { inner: body });
            Ok(resp)
        })
    }
}

impl<W> Clone for BisonService<W> {
    fn clone(&self) -> Self {
        Self {
            bison: self.bison.clone(),
        }
    }
}

pub struct BisonHttpBody {
    inner: bison::http::Body,
}

impl http_body::Body for BisonHttpBody {
    type Data = bison::http::Bytes;
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

    fn size_hint(&self) -> http_body::SizeHint {
        let (lower, upper) = self.inner.size_hint();

        let mut hint = http_body::SizeHint::new();
        hint.set_lower(lower as _);
        if let Some(upper) = upper {
            hint.set_upper(upper as _);
        }

        hint
    }
}

pub trait ToSocketAddr {
    fn to_socket_addr(self) -> io::Result<SocketAddr>;
}

impl ToSocketAddr for SocketAddr {
    fn to_socket_addr(self) -> io::Result<SocketAddr> {
        Ok(self)
    }
}

macro_rules! to_socket_addr {
    ($($ty:ty),*) => {$(
        impl ToSocketAddr for $ty {
            fn to_socket_addr(self) -> io::Result<SocketAddr> {
                Ok(self.to_socket_addrs()?.next().unwrap())
            }
        }
    )*}
}

to_socket_addr! {
    &str,
    String,
    (&str, u16),
    (IpAddr, u16),
    (String, u16),
    (Ipv4Addr, u16),
    (Ipv6Addr, u16),
    SocketAddrV4,
    SocketAddrV6
}
