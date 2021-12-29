//! Extract context from a request.

mod body;
mod default;
mod form;
mod path;
mod query;
mod state;
mod transform;

pub mod arg;

pub use body::{body, BodyConfig, BodyRejection, FromBytes};
pub use default::{default, DefaultRejection};
pub use form::{form, FormConfig, FormRejection};
pub use path::{path, FromPath, PathRejection};
pub use query::{query, FromQuery, QueryRejection};
pub use state::{state, StateRejection};
pub use transform::{Optional, Transform};

crate::util::cfg_json! {
    mod json;
    pub use json::{json, JsonRejection, JsonConfig};
}

pub(crate) fn setup(req: &mut crate::Request) {
    req.extensions_mut().insert(query::CachedQuery::default());
}
