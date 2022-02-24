use super::Request;
use crate::bounded::BoxError;
use crate::http::{FromStr, Status};
use crate::{Reject, Respond, Response};

use std::fmt;

impl Request {
    pub fn param<'r, K, V>(&'r self, keys: K) -> Result<V, K::Err>
    where
        K: ParamKeys<'r, V>,
    {
        keys.get(&self.params)
    }
}

pub trait ParamKeys<'a, Values> {
    type Err: Reject;

    fn get(self, params: &'a [(String, String)]) -> Result<Values, Self::Err>;
}

macro_rules! param_keys {
    ($( ( $( $(@$k:tt)? ($K:ident, $V:ident) ),* ), )*) => {$(
        #[allow(unused_parens)]
        impl<'a, $($V),*> ParamKeys<'a, ($($V),*)> for ($( $(@$k:tt)? &'a str),*)
        where
            $(
                $V: FromStr<'a>,
                $V::Err: Into<BoxError>
            ),*
        {
            type Err = ParamRejection;

            fn get(self, params: &'a [(String, String)]) -> Result<($($V),*), Self::Err> {
                let ($($K),*) = self;

                Ok(($({
                    let raw = params
                        .iter()
                        .find(|(key, _)| key == $K)
                        .map(|(_, value)| value.as_str())
                        .ok_or(ParamRejection::new($K, ParamRejectionKind::NotFound))?;

                    $V::from_str(raw).map_err(|e| ParamRejection::new($K, ParamRejectionKind::Parse(e.into())))?
                }),*))
            }
        }
    )*}
}

param_keys! {
    ((A, B)),
    ((A, B), (C, D)),
    ((A, B), (C, D), (E, F)),
    ((A, B), (C, D), (E, F), (G, H)),
    ((A, B), (C, D), (E, F), (G, H), (I, J)),
}

#[derive(Debug)]
pub struct ParamRejection {
    name: String,
    kind: ParamRejectionKind,
}

impl ParamRejection {
    pub fn new(name: &str, kind: ParamRejectionKind) -> Self {
        Self {
            name: name.to_owned(),
            kind,
        }
    }
}

#[derive(Debug)]
enum ParamRejectionKind {
    NotFound,
    Parse(BoxError),
}

impl Reject for ParamRejection {
    fn reject(self) -> Response {
        match self.kind {
            ParamRejectionKind::NotFound => Status::NotFound,
            ParamRejectionKind::Parse(_) => Status::BadRequest,
        }
        .respond()
    }
}

impl fmt::Display for ParamRejection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            ParamRejectionKind::NotFound => write!(f, "Expected route parameter '{}'", self.name),
            ParamRejectionKind::Parse(e) => write!(f, "{}", e),
        }
    }
}
