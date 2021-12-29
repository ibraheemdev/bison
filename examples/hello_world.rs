use bison::extract::{path, query, state, transform::Optional};
use bison::{Bison, Context};

struct State;

#[derive(Context)]
struct Hello<'req> {
    name: usize,
    bar: Optional<&'req str>,
    #[cx(state)]
    state: &'req State,
}

async fn hello(cx: Hello<'_>) -> String {
    format!("Name: {}, Bar: {:?}", cx.name, cx.bar)
}

fn main() {
    let _bison = Bison::new().get("/hello/:name", hello).inject(State);
}
