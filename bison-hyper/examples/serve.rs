use std::convert::Infallible;
use std::net::SocketAddr;

use bison::http::{Body, Response};
use bison::{Bison, Context};
use bison_hyper::{make, Server};

#[derive(Context)]
struct Hello<'req> {
    name: &'req str,
    age: &'req str,
}

async fn hello(cx: Hello<'_>) -> Result<Response, Infallible> {
    let b = format!("Hello, {} year old named {}!", cx.age, cx.name);
    Ok(Response::new(Body::once(b)))
}

#[tokio::main]
async fn main() {
    let bison = Bison::new().get("/hello/:name/:age", hello);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let server = Server::bind(&addr).serve(make(bison));

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
