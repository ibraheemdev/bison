mod bison;
mod responder;
mod router;
mod state;
mod util;

pub mod bounded;
pub mod extract;
pub mod handler;
pub mod http;
pub mod reject;
pub mod wrap;

doc_inline! {
    pub use self::http::{Request, Response};
    pub use self::wrap::Wrap;
    pub use self::bison::Bison;
    pub use self::handler::{Context};
    pub use self::reject::{Rejection, Reject};
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
