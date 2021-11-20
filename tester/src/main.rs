use core::marker::PhantomData;
use std::future::Future;

fn main() {}

struct Request<'a> {
    state: &'a String,
}

struct Response;

#[async_trait::async_trait(?Send)]
trait Handler<'a> {
    async fn handle(&self, req: Request<'a>) -> Response;
}

trait FromRequest<'a> {
    fn from_req(req: Request<'a>) -> Self;
}

#[async_trait::async_trait(?Send)]
trait FnHandler<R> {
    async fn call(&self, r: R) -> Response;
}

#[async_trait::async_trait(?Send)]
impl<F, R, Fut> FnHandler<R> for F
where
    F: Fn(R) -> Fut,
    Fut: Future<Output = Response>,
    R: 'static,
{
    async fn call(&self, r: R) -> Response {
        self(r).await
    }
}

trait RConstructor<'a> {
    type Ty: FromRequest<'a>;
}

struct FnHandlerWrapper<F, R> {
    f: F,
    _r: PhantomData<R>,
}

#[async_trait::async_trait(?Send)]
impl<'a, F, R> Handler<'a> for FnHandlerWrapper<F, R>
where
    R: RConstructor<'a>,
    F: FnHandler<R::Ty>,
{
    async fn handle(&self, req: Request<'a>) -> Response {
        self.f.call(R::Ty::from_req(req)).await
    }
}

struct App {
    routes: Vec<Box<dyn for<'a> Handler<'a>>>,
}

impl App {
    pub async fn route<F, R, Fut>(&mut self, f: F)
    where
        R: for<'a> RConstructor<'a> + 'static,
        F: for<'a> Fn(<R as RConstructor<'a>>::Ty) -> Fut + 'static,
        Fut: Future<Output = Response>,
    {
        self.routes.push(Box::new(FnHandlerWrapper {
            f,
            _r: PhantomData::<R>,
        }));
    }
}
