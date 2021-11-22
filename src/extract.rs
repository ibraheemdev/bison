use crate::error::ResponseError;
use crate::http::{self, Body, Request, Response, ResponseBuilder, StatusCode};
use crate::state::{self, State};

use std::convert::Infallible;
use std::fmt;
use std::marker::PhantomData;

pub trait Has<T> {
    fn get(&mut self) -> T;
    fn field_name(&self) -> &'static str;
}

pub struct Param<T> {
    field_name: &'static str,
    val: Option<T>,
}

impl<T> Param<T> {
    pub fn new(field_name: &'static str, val: T) -> Self {
        Self {
            field_name,
            val: Some(val),
        }
    }
}

impl<T> Has<T> for Param<T> {
    fn get(&mut self) -> T {
        self.val.take().unwrap()
    }

    fn field_name(&self) -> &'static str {
        self.field_name
    }
}

impl<T> Has<Option<T>> for Param<T> {
    fn get(&mut self) -> Option<T> {
        self.val.take()
    }

    fn field_name(&self) -> &'static str {
        self.field_name
    }
}

pub struct NoParam<T> {
    field_name: &'static str,
    _t: PhantomData<T>,
}

impl<T> NoParam<T> {
    pub fn new(field_name: &'static str) -> Self {
        Self {
            field_name,
            _t: PhantomData,
        }
    }
}

impl<T> Has<Option<T>> for NoParam<T> {
    fn get(&mut self) -> Option<T> {
        None
    }

    fn field_name(&self) -> &'static str {
        self.field_name
    }
}

impl<T> Has<()> for NoParam<T> {
    fn get(&mut self) -> () {}

    fn field_name(&self) -> &'static str {
        self.field_name
    }
}

// pub fn default<'r, T>(
//     req: &'r Request,
//     mut param: impl Has<()>,
// ) -> Result<T, ParamError<'r, T::Error>>
// where
//     T: FromParam<'r> + FromQuery<'r>,
// {
// }

pub fn param<'r, T>(
    req: &'r Request,
    mut param: impl Has<Option<&'static str>>,
) -> Result<T, ParamError<'r, T::Error>>
where
    T: FromParam<'r>,
{
    let name = param.get().unwrap_or(param.field_name());
    let params = req.extensions().get::<http::Params>().unwrap();
    let param = params.get(name).ok_or(ParamError { error: None, name })?;
    T::from_param(param).map_err(|e| ParamError {
        error: Some(e),
        name,
    })
}

pub fn state<'r, T>(req: &'r Request, _: impl Has<()>) -> Result<&'r T, StateError>
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

pub struct ParamError<'r, E> {
    error: Option<E>,
    name: &'r str,
}

impl<'r, E> fmt::Debug for ParamError<'r, E>
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

impl<'r, E> ResponseError for ParamError<'r, E>
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

// pub trait FromQuery<'r>: Sized {
//     type Error: fmt::Debug;
//
//     fn from_param(param: &'r str) -> Result<Self, Self::Error>;
// }
