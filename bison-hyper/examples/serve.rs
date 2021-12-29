use std::net::SocketAddr;

use bison::extract::{body, path as param, state, Optional};
use bison::{Bison, Context};
use bison_hyper::{make, Server};

struct State;

#[derive(Context)]
struct Hello<'req> {
    #[cx(param)]
    name: usize,
    #[cx(param)]
    bar: Optional<&'req str>,
    #[cx(state)]
    state: &'req State,
    #[cx(body)]
    body: String,
}

async fn hello(cx: Hello<'_>) -> String {
    format!("Name: {}, Bar: {:?} Body: {}", cx.name, cx.bar, cx.body)
}

#[tokio::main]
async fn main() {
    let bison = Bison::new().get("/hello/:name", hello).inject(State);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    Server::bind(&addr).serve(make(bison)).await.unwrap()
}
