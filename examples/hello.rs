use bison::{Bison, Request, Respond};

async fn hello(req: Request) -> bison::Result {
    let (age, name): (usize, &str) = req.param(("age", "name"))?;

    Ok(format!("Hello, {} year old named {}!", age, name).respond())
}

fn main() {
    Bison::new().get("/hello/:name/:age", hello);
}