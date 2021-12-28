mod bison;
mod context;
mod handler;
mod responder;
mod router;
mod scope;
mod state;
mod util;
mod wrap;

pub mod bounded;
pub mod error;
pub mod extract;
pub mod http;

pub use self::bison::Bison;
pub use self::context::{Context, WithContext};
pub use self::error::{Error, IntoResponseError, ResponseError};
pub use self::handler::Handler;
pub use self::responder::Responder;
pub use self::scope::Scope;
pub use self::state::State;
pub use self::wrap::{Next, Wrap};
pub use bison_codegen::Context;

#[doc(hidden)]
pub use self::wrap::__internal_wrap_fn;

#[doc(inline)]
pub use self::http::{Request, Response};

/// A macro for async-trait methods.
///
/// See [`async_trait`](https://docs.rs/async-trait/latest/async_trait/) for details.
pub use self::bounded::async_trait;
pub(crate) use bounded::async_trait_internal;
