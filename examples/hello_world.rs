use bison::extract::path;
use bison::{Bison, Context};
use bison_hyper::Serve;

#[derive(Context)]
struct Hello<'req> {
    name: &'req str,
    age: u8,
}

async fn hello(cx: Hello<'_>) -> String {
    format!("Hello, {} year old named {}!", cx.age, cx.name)
}

#[tokio::main]
async fn main() {
    Bison::new()
        .get("/hello/:name/:age", hello)
        .serve("localhost:3000")
        .await
        .expect("serve failed")
}
