#![deny(unsafe_op_in_unsafe_fn)]

mod bison;
mod respond;
mod router;
mod state;
mod util;

pub mod bounded;
pub mod extract;
pub mod handler;
pub mod http;
pub mod reject;
pub mod wrap;

util::doc_inline! {
    pub use self::http::{Request, Response};
    pub use self::wrap::Wrap;
    pub use self::bison::Bison;
    pub use self::handler::{Context, Handler};
    pub use self::reject::{Rejection, Reject};
    pub use self::respond::Respond;
    pub use self::router::Scope;
    pub use self::state::State;
    pub use bison_codegen::Context;
}

/// A macro for async-trait methods.
///
/// See [`async_trait`](https://docs.rs/async-trait/latest/async_trait/) for details.
pub use self::bounded::async_trait;
pub(crate) use bounded::async_trait_internal;
