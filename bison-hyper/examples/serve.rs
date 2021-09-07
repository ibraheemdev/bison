// use std::convert::Infallible;
// use std::net::SocketAddr;
// 
// use bison::{Bison, Body, Bytes, Request, Response};
// use bison_hyper::{BisonMakeService, Server};
// 
// #[tokio::main]
// async fn main() {
//     let mut bison = Bison::new();
//     bison.get("/hello_world", |_: Request| async move {
//         Ok::<_, Infallible>(Response::new(Body::once(Bytes::from("Hello world!"))))
//     });
// 
//     let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
//     let service = BisonMakeService::new(bison);
//     let server = Server::bind(&addr).serve(service);
// 
//     if let Err(e) = server.await {
//         eprintln!("server error: {}", e);
//     }
// }
//
fn main() {}
