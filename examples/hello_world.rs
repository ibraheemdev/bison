use bison::{extract::Optional, wrap_fn, Bison, Context};
use bison_hyper::Serve;

#[derive(Context)]
struct Hello {
    age: u8,
    name: Optional<String>,
}

async fn hello(cx: Hello) -> String {
    format!(
        "Hello, {} year old named {}!",
        cx.age,
        cx.name.into_inner().unwrap_or_default()
    )
}

#[tokio::main]
async fn main() {
    Bison::new()
        .get("/hello/:name/:age", hello)
        .wrap(wrap_fn!(async |req, next| next.call(req).await))
        .serve("localhost:3000")
        .await
        .expect("serve failed")
}
