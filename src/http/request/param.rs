use super::Request;
use crate::bounded::BoxError;
use crate::http::{ByteStr, FromByteStr, Status};
use crate::{Reject, Respond, Response};

use std::fmt;

impl Request {
    pub fn param<'r, P>(&'r self, keys: P::Keys) -> Result<P, P::Err>
    where
        P: FromParam<'r>,
    {
        P::get(keys, &self.params)
    }
}

pub trait FromParam<'a>: Sized {
    type Keys;
    type Err: Reject;

    fn get(keys: Self::Keys, params: &'a [(ByteStr, ByteStr)]) -> Result<Self, Self::Err>;
}

parse_params! {
    ((A, B)),
    ((A, B), (C, D)),
    ((A, B), (C, D), (E, F)),
    ((A, B), (C, D), (E, F), (G, H)),
    ((A, B), (C, D), (E, F), (G, H), (I, J)),
}

/// Error returned by ``.
#[derive(Debug)]
pub struct ParamRejection {
    name: String,
    kind: ParamRejectionKind,
}

impl ParamRejection {
    fn new(name: &str, kind: ParamRejectionKind) -> Self {
        Self {
            name: name.to_owned(),
            kind,
        }
    }
}

#[derive(Debug)]
enum ParamRejectionKind {
    NotFound,
    Parse,
}

impl Reject for ParamRejection {
    fn reject(self) -> Response {
        match self.kind {
            ParamRejectionKind::NotFound => Status::NotFound,
            ParamRejectionKind::Parse => Status::BadRequest,
        }
        .respond()
    }
}

impl fmt::Display for ParamRejection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            ParamRejectionKind::NotFound => write!(f, "Expected route parameter '{}'", self.name),
            ParamRejectionKind::Parse => write!(f, "Error parsing route parameter '{}'", self.name),
        }
    }
}

macro_rules! parse_params {
    ($( ( $( $(@$k:tt)? ($K:ident, $V:ident) ),* ), )*) => {$(
        #[allow(unused_parens, non_snake_case)]
        impl<'a, $($V),*> FromParam<'a> for ($($V),*)
        where
            $(
                $V: FromByteStr<'a>,
                $V::Err: Into<BoxError>
            ),*
        {
            type Keys = ($( $(@$k:tt)? &'a str),*);
            type Err = ParamRejection;

            fn get(keys: Self::Keys, params: &'a [(ByteStr, ByteStr)]) -> Result<($($V),*), Self::Err> {
                let ($($K),*) = keys;

                Ok(($({
                    let raw = params
                        .iter()
                        .find(|(key, _)| key == $K)
                        .map(|(_, value)| value)
                        .ok_or(ParamRejection::new($K, ParamRejectionKind::NotFound))?;

                    $V::from_byte_str(raw).map_err(|_| ParamRejection::new($K, ParamRejectionKind::Parse))?
                }),*))
            }
        }
    )*}
}

pub(self) use parse_params;
