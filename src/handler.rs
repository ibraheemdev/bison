use crate::reject::IntoRejection;
use crate::respond::Respond;

pub trait Handler<C> {
    type Response: Respond;
    type Rejection: IntoRejection;

    fn call(&self, context: C) -> Result<Self::Response, Self::Rejection>;
}
