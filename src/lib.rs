#![deny(rust_2018_idioms)]

pub mod bounded;
pub mod http;
pub mod reject;
pub mod wrap;

mod bison;
mod handler;
mod router;
mod state;
mod util;

pub use self::http::{Request, Response};
pub use bison::Bison;
pub use handler::Handler;
pub use reject::{Reject, Rejection};
pub use router::Scope;
pub use state::State;
pub use wrap::Wrap;

pub type Result<T = Response> = std::result::Result<T, Rejection>;
