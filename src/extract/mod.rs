mod body;
mod default;
mod path;
mod query;
mod state;

pub mod arg;
pub mod transform;

pub use default::{default, DefaultError};
pub use path::{path, FromPath, PathError};
pub use query::{query, FromQuery, QueryError};
pub use state::{state, StateError};
pub use transform::Transform;

pub(crate) fn setup(req: &mut crate::Request) {
    req.extensions_mut().insert(query::CachedQuery::default());
}
