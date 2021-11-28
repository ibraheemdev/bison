use crate::error::ResponseError;
use crate::http::{self, Body, Request, Response, ResponseBuilder, StatusCode};
use crate::state::{self, State};

use std::convert::Infallible;
use std::fmt;

pub struct OptionalArg<T> {
    pub value: Option<T>,
    pub field_name: &'static str,
}

impl<T> From<Arg<T>> for OptionalArg<T> {
    fn from(arg: Arg<T>) -> Self {
        Self {
            value: Some(arg.value),
            field_name: arg.field_name,
        }
    }
}

impl<T> From<NoArg> for OptionalArg<T> {
    fn from(arg: NoArg) -> Self {
        Self {
            value: None,
            field_name: arg.field_name,
        }
    }
}

pub struct Arg<T> {
    pub field_name: &'static str,
    pub value: T,
}

impl<T> Arg<T> {
    #[doc(hidden)]
    pub fn new(field_name: &'static str, value: T) -> Self {
        Self { field_name, value }
    }
}

pub struct NoArg {
    pub field_name: &'static str,
}

impl NoArg {
    #[doc(hidden)]
    pub fn new(field_name: &'static str) -> Self {
        Self { field_name }
    }
}

pub fn default<'req, T>(req: &'req Request, arg: NoArg) -> Result<T, DefaultError>
where
    T: FromParam<'req> + FromQuery<'req>,
{
    param(req, NoArg::new(arg.field_name).into())
        .or_else(|_| query(req, arg))
        .map_err(|_| DefaultError {
            ty: std::any::type_name::<T>(),
        })
}

pub fn param<'req, T>(
    req: &'req Request,
    param: OptionalArg<&'static str>,
) -> Result<T, ParamError<T::Error>>
where
    T: FromParam<'req>,
{
    let name = param.value.unwrap_or(param.field_name);
    let params = req.extensions().get::<http::Params>().unwrap();
    let param = params.get(name).ok_or(ParamError {
        error: None,
        name: name.to_owned(),
    })?;
    T::from_param(param).map_err(|e| ParamError {
        error: Some(e),
        name: name.to_owned(),
    })
}

pub fn query<'req, T>(req: &'req Request, _: NoArg) -> Result<T, QueryError>
where
    T: FromQuery<'req>,
{
    let query = req.uri().query().unwrap_or_default();
    serde_urlencoded::from_str(query).map_err(|error| QueryError {
        error,
        t: std::any::type_name::<T>(),
    })
}

pub fn state<'req, T>(req: &'req Request, _: NoArg) -> Result<&'req T, StateError>
where
    T: State,
{
    req.extensions()
        .get::<state::Map>()
        .unwrap()
        .get::<T>()
        .ok_or(StateError)
}

pub struct StateError;

#[derive(Debug)]
pub struct ParamError<E> {
    error: Option<E>,
    name: String,
}

impl<E> fmt::Display for ParamError<E>
where
    E: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.error {
            Some(err) => write!(f, "error extracting param '{}': {}", self.name, err),
            None => write!(f, "param '{}' not found", self.name),
        }
    }
}

impl<E> ResponseError for ParamError<E>
where
    E: fmt::Debug + fmt::Display + Send + Sync,
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

impl<'req> FromParam<'req> for &'req str {
    type Error = Infallible;

    fn from_param(param: &'req str) -> Result<Self, Self::Error> {
        Ok(param)
    }
}

pub trait FromQuery<'req>: serde::Deserialize<'req> {}
impl<'req, T> FromQuery<'req> for T where T: serde::Deserialize<'req> {}

#[derive(Debug)]
pub struct QueryError {
    error: serde_urlencoded::de::Error,
    t: &'static str,
}

impl fmt::Display for QueryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "failed to deserialize `{}` from query string: {}",
            self.t, self.error,
        )
    }
}

impl ResponseError for QueryError {
    fn respond(&mut self) -> Response {
        ResponseBuilder::new()
            .status(StatusCode::NOT_FOUND)
            .body(Body::empty())
            .unwrap()
    }
}

#[derive(Debug)]
pub struct DefaultError {
    ty: &'static str,
}

impl fmt::Display for DefaultError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "failed to extract `{}` url parameter or query string",
            self.ty
        )
    }
}

impl ResponseError for DefaultError {
    fn respond(&mut self) -> Response {
        ResponseBuilder::new()
            .status(StatusCode::NOT_FOUND)
            .body(Body::empty())
            .unwrap()
    }
}
