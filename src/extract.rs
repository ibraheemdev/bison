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

pub fn default<'r, T>(req: &'r Request, arg: NoArg) -> Result<T, DefaultError>
where
    T: FromParam<'r> + FromQuery<'r>,
{
    param(req, NoArg::new(arg.field_name).into())
        .or_else(|_| query(req, arg))
        .map_err(|_| DefaultError {
            ty: std::any::type_name::<T>(),
        })
}

pub fn param<'r, T>(
    req: &'r Request,
    param: OptionalArg<&'static str>,
) -> Result<T, ParamError<'r, T::Error>>
where
    T: FromParam<'r>,
{
    let name = param.value.unwrap_or(param.field_name);
    let params = req.extensions().get::<http::Params>().unwrap();
    let param = params.get(name).ok_or(ParamError { error: None, name })?;
    T::from_param(param).map_err(|e| ParamError {
        error: Some(e),
        name,
    })
}

pub fn query<'r, T>(req: &'r Request, _: NoArg) -> Result<T, QueryError>
where
    T: FromQuery<'r>,
{
    let query = req.uri().query().unwrap_or_default();
    serde_urlencoded::from_str(query).map_err(|error| QueryError {
        error,
        t: std::any::type_name::<T>(),
    })
}

pub fn state<'r, T>(req: &'r Request, _: NoArg) -> Result<&'r T, StateError>
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
pub struct ParamError<'r, E> {
    error: Option<E>,
    name: &'r str,
}

impl<'r, E> fmt::Display for ParamError<'r, E>
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

impl<'r, E> ResponseError for ParamError<'r, E>
where
    E: fmt::Debug + fmt::Display,
{
    fn respond(&mut self) -> Response {
        ResponseBuilder::new()
            .status(StatusCode::NOT_FOUND)
            .body(Body::empty())
            .unwrap()
    }
}

pub trait FromParam<'r>: Sized {
    type Error: fmt::Debug;

    fn from_param(param: &'r str) -> Result<Self, Self::Error>;
}

impl<'r> FromParam<'r> for &'r str {
    type Error = Infallible;

    fn from_param(param: &'r str) -> Result<Self, Self::Error> {
        Ok(param)
    }
}

pub trait FromQuery<'r>: serde::Deserialize<'r> {}
impl<'r, T> FromQuery<'r> for T where T: serde::Deserialize<'r> {}

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
