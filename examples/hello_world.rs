use bison::{Bison, Context};

#[derive(Context)]
struct Hello<'req> {
    name: &'req str,
}

async fn hello(cx: Hello<'_>) -> String {
    format!("Hello {}!", cx.name)
}

fn main() {
    let _bison = Bison::new().get("/hello/:name", hello);
}
