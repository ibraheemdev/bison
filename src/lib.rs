mod bison;
mod context;
mod handler;
mod responder;
mod router;
mod state;
mod util;

pub mod bounded;
pub mod error;
pub mod extract;
pub mod http;
pub mod wrap;

doc_inline! {
    pub use self::http::{Request, Response};
    pub use self::wrap::Wrap;
    pub use self::bison::Bison;
    pub use self::context::{Context, WithContext};
    pub use self::error::{Error, IntoResponseError, ResponseError};
    pub use self::handler::Handler;
    pub use self::responder::Responder;
    pub use self::router::Scope;
    pub use self::state::State;
    pub use bison_codegen::Context;
}

/// A macro for async-trait methods.
///
/// See [`async_trait`](https://docs.rs/async-trait/latest/async_trait/) for details.
pub use self::bounded::async_trait;
pub(crate) use bounded::async_trait_internal;

macro_rules! doc_inline {
    ($($x:item)*) => {$(
        #[doc(inline)]
        $x
    )*}
}

use doc_inline;
