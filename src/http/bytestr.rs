use std::borrow::Borrow;
use std::fmt;

/// A UTF-8 encoded string stored as [`Bytes`].
#[derive(Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ByteStr(bytes::Bytes);

impl ByteStr {
    pub fn new(str: impl Into<ByteStr>) -> ByteStr {
        str.into()
    }

    pub fn from_static(str: &'static str) -> ByteStr {
        ByteStr(bytes::Bytes::from_static(str.as_bytes()))
    }

    pub fn as_str(&self) -> &str {
        self
    }
}

impl std::ops::Deref for ByteStr {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        // SAFETY: always valid UTF-8
        unsafe { std::str::from_utf8_unchecked(&self.0) }
    }
}

impl fmt::Debug for ByteStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.as_str())
    }
}

impl fmt::Display for ByteStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl AsRef<str> for ByteStr {
    fn as_ref(&self) -> &str {
        &*self
    }
}

impl Borrow<str> for ByteStr {
    fn borrow(&self) -> &str {
        &*self
    }
}

impl PartialEq<str> for ByteStr {
    fn eq(&self, other: &str) -> bool {
        &*self == other
    }
}

impl PartialEq<ByteStr> for str {
    fn eq(&self, other: &ByteStr) -> bool {
        self == &*other
    }
}

impl From<String> for ByteStr {
    fn from(string: String) -> Self {
        ByteStr(string.into())
    }
}

impl From<&str> for ByteStr {
    fn from(str: &str) -> Self {
        ByteStr(str.to_owned().into())
    }
}
