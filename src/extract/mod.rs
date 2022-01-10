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

pub async fn nest<T: crate::Context>(req: &crate::Request, _: ()) -> Result<T, crate::Rejection> {
    <T as crate::Context>::extract(req.clone()).await
}
