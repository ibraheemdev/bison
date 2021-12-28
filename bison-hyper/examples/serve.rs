use std::net::SocketAddr;

use bison::extract::transform;
use bison::{wrap_fn, Bison, Context, Error, Next, Request, Response, Wrap};
use bison_hyper::{make, Server};

#[derive(Context)]
struct Hello<'req> {
    name: usize,
    baz: transform::Option<usize>,
    bar: transform::Option<&'req str>,
}

async fn hello(cx: Hello<'_>) -> String {
    format!("Name: {}, Bar: {:?}, Baz: {:?}", cx.name, cx.bar, cx.baz)
}

#[tokio::main]
async fn main() {
    let bison = Bison::new()
        .get("/hello/:name", hello)
        .wrap(Logger)
        .wrap(wrap_fn!(|req, next| {
            next.call(req).await.map_err(|err| {
                eprintln!("{}", err);
                err
            })
        }));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    Server::bind(&addr).serve(make(bison)).await.unwrap()
}

struct Logger;

#[bison::async_trait]
impl Wrap for Logger {
    type Error = Error;

    async fn call<'a>(&self, req: &Request, next: impl Next + 'a) -> Result<Response, Self::Error> {
        match next.call(req).await {
            Ok(res) => Ok(res),
            Err(err) => {
                eprintln!("{}", err);
                Err(err)
            }
        }
    }
}
