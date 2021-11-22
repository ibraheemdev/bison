use bison::extract::*;
use bison::Context;

#[derive(Context)]
struct Foo<'r> {
    #[cx(param)]
    id: &'r str,
}

struct Database;

fn main() {}
