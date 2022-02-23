use super::IntoHeader;
use crate::http::ByteStr;

use std::collections::hash_map::{self, HashMap};
use std::{fmt, iter, mem};

/// A map of HTTP headers.
///
/// Header names are compared case-insensitively.
pub struct Headers {
    map: HashMap<CaseInsensitive, HeaderValue>,
}

impl Headers {
    /// Create an empty collection of headers.
    pub fn new() -> Headers {
        Headers {
            map: HashMap::with_capacity(16),
        }
    }

    /// Returns `true` if a header exists with the given name.
    pub fn contains(&self, name: &str) -> bool {
        self.map.contains_key(CaseInsensitiveStr::new(name))
    }

    /// The number of headers in this map.
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Returns `true` if this map is empty.
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Returns an iterator over all values of the header with
    /// the given name.
    pub fn get<'a>(&'a self, name: &str) -> Values<'_> {
        self.map
            .get(CaseInsensitiveStr::new(name))
            .map(|values| values.iter())
            .unwrap_or(Values {
                kind: ValuesKind::None,
            })
    }

    /// Insert a header into the map.
    ///
    /// If a header with the same name already exists, it will
    /// be overwritten and `true` will be returned.
    pub fn insert<H>(&mut self, header: H) -> bool
    where
        H: IntoHeader,
    {
        let (name, values) = header.into_header();

        self.map
            .insert(CaseInsensitive(name), HeaderValue::from_iter(values))
            .is_some()
    }

    /// Append a header to the map.
    ///
    /// If a header with the same name already exists, the
    /// new value will be appended.
    pub fn append<H>(&mut self, header: H)
    where
        H: IntoHeader,
    {
        let (name, values) = header.into_header();

        match self.map.entry(CaseInsensitive(name)) {
            hash_map::Entry::Occupied(mut entry) => match entry.get_mut() {
                HeaderValue::One(old) => {
                    let old = mem::take(old);

                    let new = match HeaderValue::from_iter(values) {
                        HeaderValue::One(value) => HeaderValue::Many(vec![old, value]),
                        HeaderValue::Many(mut values) => {
                            values.insert(0, old);
                            HeaderValue::Many(values)
                        }
                    };

                    entry.insert(new);
                }
                HeaderValue::Many(old) => {
                    old.extend(values);
                }
            },
            hash_map::Entry::Vacant(entry) => {
                entry.insert(HeaderValue::from_iter(values));
            }
        }
    }

    /// Remove a header from the map.
    ///
    /// Returns an iterator over the values of the header
    /// that was removed.
    pub fn remove(&mut self, name: &str) -> Removed {
        let kind = self
            .map
            .remove(CaseInsensitiveStr::new(name))
            .map(|value| match value {
                HeaderValue::One(value) => RemovedKind::One(iter::once(value)),
                HeaderValue::Many(values) => RemovedKind::Many(values.into_iter()),
            })
            .unwrap_or(RemovedKind::None);

        Removed { kind }
    }

    /// An iterator over headers in this map.
    pub fn iter(&self) -> Iter<'_> {
        Iter {
            map: self.map.iter(),
            current: None,
        }
    }
}

impl Default for Headers {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for Headers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.map)
    }
}

/// An iterator over HTTP headers.
///
/// See [`Headers::iter`] for details.
pub struct Iter<'a> {
    map: hash_map::Iter<'a, CaseInsensitive, HeaderValue>,
    current: Option<(&'a CaseInsensitive, Values<'a>)>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = (&'a ByteStr, &'a ByteStr);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some((name, ref mut values)) = self.current {
                if let Some(value) = values.next() {
                    return Some((&name.0, value));
                }

                self.current = None;
            }

            match self.map.next() {
                Some((name, values)) => {
                    self.current = Some((name, values.iter()));
                }
                None => return None,
            }
        }
    }
}

/// An iterator over all values of a given header.
///
/// See [`Headers::get`] for details.
pub struct Values<'a> {
    kind: ValuesKind<'a>,
}

enum ValuesKind<'a> {
    None,
    One(iter::Once<&'a ByteStr>),
    Many(std::slice::Iter<'a, ByteStr>),
}

impl<'a> Iterator for Values<'a> {
    type Item = &'a ByteStr;

    fn next(&mut self) -> Option<Self::Item> {
        match self.kind {
            ValuesKind::None => return None,
            ValuesKind::One(ref mut o) => o.next(),
            ValuesKind::Many(ref mut m) => m.next(),
        }
    }
}

/// An iterator over the values of a removed header.
///
/// See [`Headers::remove`] for details.
pub struct Removed {
    kind: RemovedKind,
}

enum RemovedKind {
    None,
    One(iter::Once<ByteStr>),
    Many(std::vec::IntoIter<ByteStr>),
}

impl Iterator for Removed {
    type Item = ByteStr;

    fn next(&mut self) -> Option<Self::Item> {
        match self.kind {
            RemovedKind::None => None,
            RemovedKind::One(ref mut o) => o.next(),
            RemovedKind::Many(ref mut m) => m.next(),
        }
    }
}

/// Internal representation of HTTP header values.
enum HeaderValue {
    One(ByteStr),
    Many(Vec<ByteStr>),
}

impl HeaderValue {
    fn from_iter<I>(iter: I) -> HeaderValue
    where
        I: IntoIterator<Item = ByteStr>,
    {
        let mut iter = iter.into_iter();
        let first = iter.next().expect("expected at least one header value");

        match iter.next() {
            Some(second) => {
                let values = iter
                    .chain(iter::once(first))
                    .chain(iter::once(second))
                    .collect();
                HeaderValue::Many(values)
            }
            None => HeaderValue::One(first),
        }
    }

    fn iter(&self) -> Values<'_> {
        let kind = match self {
            HeaderValue::One(value) => ValuesKind::One(iter::once(value)),
            HeaderValue::Many(values) => ValuesKind::Many(values.iter()),
        };

        Values { kind }
    }
}

impl fmt::Debug for HeaderValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_set().entries(self.iter()).finish()
    }
}

use case_insensitive::{CaseInsensitive, CaseInsensitiveStr};

mod case_insensitive {
    use super::*;

    use std::borrow::Borrow;
    use std::hash::{Hash, Hasher};

    #[repr(transparent)]
    pub(super) struct CaseInsensitiveStr(str);

    impl CaseInsensitiveStr {
        pub(super) fn new(str: &str) -> &CaseInsensitiveStr {
            // SAFETY: repr(transparent)
            unsafe { &*(str as *const _ as *const _) }
        }
    }

    impl PartialEq for CaseInsensitiveStr {
        fn eq(&self, other: &CaseInsensitiveStr) -> bool {
            self.0.eq_ignore_ascii_case(&other.0)
        }
    }

    impl Eq for CaseInsensitiveStr {}

    impl Hash for CaseInsensitiveStr {
        fn hash<H: Hasher>(&self, hasher: &mut H) {
            for byte in self.0.bytes() {
                hasher.write_u8(byte.to_ascii_lowercase());
            }
        }
    }

    impl fmt::Debug for CaseInsensitiveStr {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{:?}", &self.0)
        }
    }

    impl fmt::Display for CaseInsensitiveStr {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", &self.0)
        }
    }

    pub(super) struct CaseInsensitive(pub(super) ByteStr);

    impl Borrow<CaseInsensitiveStr> for CaseInsensitive {
        fn borrow(&self) -> &CaseInsensitiveStr {
            CaseInsensitiveStr::new(&self.0)
        }
    }

    impl PartialEq for CaseInsensitive {
        fn eq(&self, other: &CaseInsensitive) -> bool {
            self.0.eq_ignore_ascii_case(&other.0)
        }
    }

    impl Eq for CaseInsensitive {}

    impl Hash for CaseInsensitive {
        fn hash<H: Hasher>(&self, hasher: &mut H) {
            for byte in self.0.bytes() {
                hasher.write_u8(byte.to_ascii_lowercase());
            }
        }
    }

    impl fmt::Debug for CaseInsensitive {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{:?}", &self.0)
        }
    }

    impl fmt::Display for CaseInsensitive {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", &self.0)
        }
    }
}
