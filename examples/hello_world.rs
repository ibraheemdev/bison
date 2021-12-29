use bison::{Bison, Context};

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
    let bison = Bison::new().get("/hello/:name/:age", hello);

    bison_hyper::serve("localhost:3000", bison)
        .await
        .expect("serve failed")
}
