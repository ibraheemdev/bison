use std::net::SocketAddr;

use bison::http::{Request, Response};
use bison::{Bison, Context, Error, HandlerExt, Next, Wrap};
use bison_hyper::{make, Server};

struct Logger;

#[bison::async_trait]
impl Wrap for Logger {
    type Error = Error;

    async fn call<'a>(&self, req: &Request, next: impl Next + 'a) -> Result<Response, Self::Error> {
        println!("{:?}", req);
        next.call(req).await
    }
}

#[derive(Context)]
struct Hello<'req> {
    name: &'req str,
}

async fn hello(cx: Hello<'_>) -> String {
    format!("Hello {}!", cx.name)
}

async fn get_users() -> String {
    String::new()
}

async fn create_user() -> String {
    String::new()
}

async fn update_user() -> String {
    String::new()
}

#[tokio::main]
async fn main() {
    let bison = Bison::new()
        .get("/hello/:name", hello)
        .wrap(Logger)
        .scope("/api", |s| {
            s.get("/users/", get_users.wrap(Logger))
                .post("/users/", create_user)
                .put("/user/:id", update_user.wrap(Logger))
                .wrap(bison::wrap_fn(some_middleware))
                .wrap(Logger)
        });

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let server = Server::bind(&addr).serve(make(bison));

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}

async fn some_middleware(req: &Request, next: &dyn Next) -> Result<Response, bison::Error> {
    dbg!("{:?}", req.uri());
    next.call(req).await
}
