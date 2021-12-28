use bison::extract::{path, query, transform};
use bison::{Bison, Context};

#[derive(Context)]
struct Hello<'req> {
    #[cx(path = "_name")]
    name: usize,
    #[cx(query = "bar")]
    bar: transform::Option<&'req str>,
    #[cx(query = "baz")]
    baz: transform::Option<&'req str>,
}

async fn hello(cx: Hello<'_>) -> String {
    format!(
        "Name: {}, Bar: {:?}, Baz: {:?}",
        cx.name,
        cx.bar.into_inner(),
        cx.baz.into_inner()
    )
}

fn main() {
    let _bison = Bison::new().get("/hello/:_name", hello);
}
