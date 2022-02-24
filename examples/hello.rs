use bison::{Bison, Request, Respond};

async fn hello(req: Request) -> bison::Result {
    let (age, name) = req.param(("name", "age"))?;

    Ok(format!("Hello, {} year old named {}!", age, name).respond())
}

fn main() {
    Bison::new().get("/hello/:name/:age", hello);
}
