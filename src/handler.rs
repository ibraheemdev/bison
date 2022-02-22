use crate::context::Context;
use crate::http::{Request, Response};
use crate::reject::{IntoRejection, Rejection};
use crate::respond::Respond;

pub trait Handler {
    fn call(&self, request: &Request) -> crate::Result;
}
