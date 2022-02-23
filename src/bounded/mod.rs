mod not_send;
mod send;

#[cfg(feature = "not-send")]
pub use not_send::*;

#[cfg(not(feature = "not-send"))]
pub use send::*;
