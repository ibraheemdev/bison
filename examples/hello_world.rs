// use bison::{wrap_fn, Bison, Context};
// use bison_hyper::Serve;
//
// #[derive(Context)]
// struct Hello {
//     age: u8,
//     name: String,
// }
//
// async fn hello(cx: Hello) -> String {
//     format!("Hello, {} year old named {}!", cx.age, cx.name)
// }
//
// #[tokio::main]
// async fn main() {
//     Bison::new()
//         .get("/hello/:name/:age", hello)
//         .wrap(wrap_fn!(async |req, next| next.call(req).await))
//         .serve("localhost:3000")
//         .await
//         .expect("serve failed")
// }

use bison::{wrap_fn, Bison, Context};
use bison_hyper::Serve;

async fn hello() -> &'static str {
    "Hello world"
}

#[tokio::main]
async fn main() {
    Bison::new()
        .get("/", hello)
        .serve("localhost:3000")
        .await
        .expect("serve failed")
}
