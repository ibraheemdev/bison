use std::borrow::Borrow;
use std::fmt;
use std::str::Utf8Error;

use bytes::Bytes;

/// An immutable UTF-8 encoded string with [`Bytes`] as a storage.
#[derive(Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RcStr(Bytes);

impl RcStr {
    pub fn as_str(&self) -> &str {
        self
    }

    pub fn try_from_utf8(bytes: Bytes) -> Result<Self, Utf8Error> {
        std::str::from_utf8(&bytes)?;
        Ok(Self(bytes))
    }
}

impl fmt::Debug for RcStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.as_str())
    }
}

impl fmt::Display for RcStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::ops::Deref for RcStr {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        // SAFETY: always valid UTF-8
        unsafe { std::str::from_utf8_unchecked(&self.0) }
    }
}

impl From<&'static str> for RcStr {
    fn from(str: &'static str) -> Self {
        Self(str.into())
    }
}

impl From<String> for RcStr {
    fn from(str: String) -> Self {
        Self(str.into())
    }
}

impl AsRef<str> for RcStr {
    fn as_ref(&self) -> &str {
        &*self
    }
}

impl Borrow<str> for RcStr {
    fn borrow(&self) -> &str {
        &*self
    }
}

impl PartialEq<str> for RcStr {
    fn eq(&self, other: &str) -> bool {
        &*self == other
    }
}

impl PartialEq<RcStr> for str {
    fn eq(&self, other: &RcStr) -> bool {
        self == &*other
    }
}
