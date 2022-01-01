use std::borrow::Borrow;

/// An immutable UTF-8 encoded string with [`Bytes`] as a storage.
#[derive(Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RcStr(bytes::Bytes);

impl RcStr {
    pub fn as_str(&self) -> &str {
        self
    }
}

impl std::ops::Deref for RcStr {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        // SAFETY: always valid UTF-8
        unsafe { std::str::from_utf8_unchecked(&self.0) }
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
