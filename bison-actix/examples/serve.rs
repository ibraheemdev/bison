use actix_http::HttpService;
use actix_server::Server;
use bison::{Bison, HasContext, Request, Response, ResponseBuilder};
use bison_actix::BisonServiceFactory;

#[derive(Clone)]
struct State {
    planet: String,
}

#[derive(HasContext)]
struct Foo {
    #[param("baar")]
    bar: String,
    #[param]
    id: usize,
}

async fn test(context: Foo) -> Response {
    Response::text(format!("bar = {}, id = {}", context.bar, context.id))
}

async fn home(req: Request<State>) -> Response {
    Response::text(format!("Hello {}!", req.state().planet))
}

async fn planet(req: Request<State>) -> Response {
    Response::text(format!("Hello {}!", req.param("planet").unwrap()))
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let state = State {
        planet: "world".into(),
    };

    Server::build()
        .bind("hello-world", "127.0.0.1:3000", move || {
            let bison = Bison::with_state(state.clone())
                .get("/", home)
                .get("/test/:baar/:id", test)
                .get("/planet/:planet", planet)
                .get("/ping", |_: Request<_>| async { Response::text("pong") });

            HttpService::build()
                .client_timeout(1000)
                .client_disconnect(1000)
                .finish(BisonServiceFactory::new(bison))
                .tcp()
        })?
        .run()
        .await
}
