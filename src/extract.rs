use crate::error::ResponseError;
use crate::http::{self, Body, Request, Response, ResponseBuilder, StatusCode};

use std::fmt;

pub trait Params<T> {
    fn get(self) -> T;
}

pub enum None {}

fn param<'req, T>(
    req: &'req Request,
    name: impl Params<&'static str>,
) -> Result<T, ParamError<'req, T::Error>>
where
    T: FromParam<'req>,
{
    let name = name.get();
    let params = req.extensions().get::<http::Params>().unwrap();
    let param = params.get(name).ok_or(ParamError { error: None, name })?;
    T::from_param(param).map_err(|e| ParamError {
        error: Some(e),
        name,
    })
}

struct ParamError<'req, E> {
    error: Option<E>,
    name: &'req str,
}

impl<'req, E> fmt::Debug for ParamError<'req, E>
where
    E: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.error {
            Some(err) => write!(f, "error extracting param '{}': {:?}", self.name, err),
            None => write!(f, "param '{}' not found", self.name),
        }
    }
}

impl<'req, E> ResponseError for ParamError<'req, E>
where
    E: fmt::Debug,
{
    fn respond(&mut self) -> Response {
        ResponseBuilder::new()
            .status(StatusCode::NOT_FOUND)
            .body(Body::empty())
            .unwrap()
    }
}

pub trait FromParam<'req>: Sized {
    type Error: fmt::Debug;

    fn from_param(param: &'req str) -> Result<Self, Self::Error>;
}
