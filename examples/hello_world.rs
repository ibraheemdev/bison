use bison::Context;

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

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    Server::bind(&addr)
        .serve(bison_hyper::make(bison))
        .await
        .expect("server error")
}
