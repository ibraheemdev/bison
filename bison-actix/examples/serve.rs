use std::convert::Infallible;
use std::io;

use actix_http::HttpService;
use actix_server::Server;
use bison::{Bison, Body, Bytes, Request, Response};
use bison_actix::BisonServiceFactory;

#[actix_rt::main]
async fn main() -> io::Result<()> {
    Server::build()
        .bind("hello-world", "127.0.0.1:3000", || {
            let bison = Bison::new()
                .get("/hello", |_: Request<()>| async move {
                    Ok::<_, Infallible>(Response::new(Body::once(Bytes::from("Hello world!"))))
                })
                .post("/:id", |_: Request<()>| async move {
                    Ok::<_, Infallible>(Response::new(Body::once(Bytes::from("Hello world!"))))
                });

            HttpService::build()
                .client_timeout(1000)
                .client_disconnect(1000)
                .finish(BisonServiceFactory::new(bison))
                .tcp()
        })?
        .run()
        .await
}
