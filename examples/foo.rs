use bison::extract::*;
use bison::Context;

#[derive(Context)]
struct Foo<'r> {
    id: &'r str,
    something: &'r str,
    #[cx(param = "OTHER")]
    other: &'r str,
    #[cx(param)]
    otherr: &'r str,
    #[cx(query)]
    otherrr: Query,
}

#[derive(serde::Deserialize)]
struct Query {
    name: String,
    id: usize,
}

struct Database;

fn main() {}
