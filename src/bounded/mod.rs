cfg_not_send! {
    mod not_send;
    pub use not_send::*;
}

cfg_send! {
    mod send;
    pub use send::*;
}

macro_rules! cfg_send {
    ($($x:item)*) => {$(
        #[cfg(not(feature = "not-send"))]
        $x
    )*}
}

macro_rules! cfg_not_send {
    ($($x:item)*) => {$(
        #[cfg(feature = "not-send")]
        $x
    )*}
}

pub(crate) use {cfg_not_send, cfg_send};
