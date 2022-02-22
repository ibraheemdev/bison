use super::{Header, IntoHeader};
use crate::http::ByteStr;

use std::collections::hash_map::{self, HashMap};
use std::{fmt, iter, mem};

pub struct Headers {
    map: HashMap<ByteStr, HeaderValue>,
}

impl Headers {
    pub fn new() -> Headers {
        Headers {
            map: HashMap::with_capacity(16),
        }
    }

    pub fn contains(&self, name: &str) -> bool {
        self.map.contains_key(name)
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn get<'a>(&'a self, name: &str) -> Values<'_> {
        self.map
            .get(name)
            .map(|values| values.iter())
            .unwrap_or(Values {
                kind: ValuesKind::None,
            })
    }

    pub fn insert<H>(&mut self, header: H) -> bool
    where
        H: IntoHeader,
    {
        let Header { name, value } = header.into_header();

        self.map.insert(name, HeaderValue::One(value)).is_some()
    }

    pub fn append<H>(&mut self, header: H)
    where
        H: IntoHeader,
    {
        let Header { name, value } = header.into_header();

        match self.map.entry(name) {
            hash_map::Entry::Occupied(mut entry) => match entry.get_mut() {
                HeaderValue::One(old) => {
                    let old = mem::take(old);
                    entry.insert(HeaderValue::Many(vec![old, value]));
                }
                HeaderValue::Many(values) => {
                    values.push(value);
                }
            },
            hash_map::Entry::Vacant(entry) => {
                entry.insert(HeaderValue::One(value));
            }
        }
    }

    pub fn remove(&mut self, name: &str) -> Removed {
        let kind = self
            .map
            .remove(name)
            .map(|value| match value {
                HeaderValue::One(value) => RemovedKind::One(iter::once(value)),
                HeaderValue::Many(values) => RemovedKind::Many(values.into_iter()),
            })
            .unwrap_or(RemovedKind::None);

        Removed { kind }
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        Iter {
            map: self.map.iter(),
            current: None,
        }
    }
}

impl fmt::Debug for Headers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.map)
    }
}

pub struct Iter<'a> {
    map: hash_map::Iter<'a, ByteStr, HeaderValue>,
    current: Option<(&'a ByteStr, Values<'a>)>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = (&'a str, &'a str);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some((name, ref mut values)) = self.current {
                if let Some(value) = values.next() {
                    return Some((name, value));
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

enum HeaderValue {
    One(ByteStr),
    Many(Vec<ByteStr>),
}

impl HeaderValue {
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

pub struct Values<'a> {
    kind: ValuesKind<'a>,
}

enum ValuesKind<'a> {
    None,
    One(iter::Once<&'a ByteStr>),
    Many(std::slice::Iter<'a, ByteStr>),
}

impl<'a> Iterator for Values<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        match self.kind {
            ValuesKind::None => return None,
            ValuesKind::One(ref mut o) => o.next(),
            ValuesKind::Many(ref mut m) => m.next(),
        }
        .map(|b| b.as_str())
    }
}

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
