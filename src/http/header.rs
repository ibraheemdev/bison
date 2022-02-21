// use crate::bounded::RefCell;
// use crate::http::RcStr;
// 
// use std::borrow::Borrow;
// use std::cmp::Ordering;
// use std::collections::{hash_map, HashMap};
// use std::hash::{Hash, Hasher};
// use std::iter;
// use std::ops::Deref;
// use std::{fmt, mem};
// 
// pub struct HeaderMap {
//     map: RefCell<HashMap<HeaderName, HeaderValue>>,
// }
// 
// impl HeaderMap {
//     pub fn new() -> HeaderMap {
//         HeaderMap {
//             map: RefCell::new(HashMap::new()),
//         }
//     }
// 
//     pub fn contains(&self, name: &str) -> bool {
//         self.map.borrow_mut().contains_key(name)
//     }
// 
//     pub fn len(&self) -> usize {
//         self.map.borrow_mut().len()
//     }
// 
//     pub fn is_empty(&self) -> bool {
//         self.len() == 0
//     }
// 
//     pub fn get<'a>(&'a self, name: &'a str) -> impl Iterator<Item = RcStr> + 'a {
//         enum Impl<'a> {
//             None,
//             Once(iter::Once<RcStr>),
//             Many {
//                 map: &'a HeaderMap,
//                 name: &'a str,
//                 index: usize,
//             },
//         }
// 
//         impl<'a> Iterator for Impl<'a> {
//             type Item = RcStr;
// 
//             fn next(&mut self) -> Option<Self::Item> {
//                 match self {
//                     Impl::None => None,
//                     Impl::Once(value) => value.next(),
//                     Impl::Many { map, name, index } => {
//                         match map.map.borrow_mut().get(*name).unwrap() {
//                             HeaderValue::Many(values) => values.get(*index).map(|value| {
//                                 *index += 1;
//                                 value.clone()
//                             }),
//                             _ => unreachable!(),
//                         }
//                     }
//                 }
//             }
//         }
// 
//         match self.map.borrow_mut().get(name).cloned() {
//             Some(HeaderValue::One(item)) => Impl::Once(iter::once(item)),
//             Some(HeaderValue::Many(_)) => Impl::Many {
//                 map: self,
//                 name,
//                 index: 0,
//             },
//             None => Impl::None,
//         }
//     }
// 
//     pub fn first(&self, name: &str) -> Option<RcStr> {
//         self.map.borrow_mut().get(name).map(|value| match value {
//             HeaderValue::One(value) => value.clone(),
//             HeaderValue::Many(values) => values.first().unwrap().clone(),
//         })
//     }
// 
//     pub fn replace<N, V>(&self, name: N, value: V)
//     where
//         N: Into<RcStr>,
//         V: Into<HeaderValue>,
//     {
//         self.map
//             .borrow_mut()
//             .insert(HeaderName(name.into()), value.into());
//     }
// 
//     pub fn add<N, V>(&self, name: N, value: V)
//     where
//         N: Into<RcStr>,
//         V: Into<HeaderValue>,
//     {
//         let name = HeaderName(name.into());
// 
//         match self.map.borrow_mut().entry(name) {
//             hash_map::Entry::Occupied(mut entry) => entry.get_mut().concat(value.into()),
//             hash_map::Entry::Vacant(entry) => {
//                 entry.insert(value.into());
//             }
//         }
//     }
// 
//     pub(crate) fn from_http(mut http_map: ::http::HeaderMap) -> Self {
//         let mut drain = http_map.drain();
//         let (first_name, first_value) = match drain.next() {
//             None => return HeaderMap::new(),
//             Some((name, value)) => {
//                 let name = RcStr::from(name.unwrap().as_str());
//                 let value = value.to_str().map(RcStr::from);
//                 (name, value)
//             }
//         };
// 
//         let (lower, upper) = drain.size_hint();
//         let capacity = upper.unwrap_or(lower);
//         let mut headers = Self {
//             map: RefCell::new(HashMap::with_capacity(lower)),
//         };
// 
//         headers.append(first_name.clone(), first_value);
// 
//         let (headers, _) = drain.fold(
//             (headers, first_name),
//             |(mut headers, prev_name), (name, value)| {
//                 let name = name
//                     .map(|name| RcStr::from(name.as_str()))
//                     .unwrap_or(prev_name);
// 
//                 if let Some(value) = value.to_str().ok().map(RcStr::from) {
//                     headers.add((name.clone(), value));
//                 }
// 
//                 (headers, name)
//             },
//         );
// 
//         headers
//     }
// }
// 
// #[derive(Eq)]
// pub struct HeaderName(RcStr);
// 
// #[derive(Clone)]
// pub enum HeaderValue {
//     One(RcStr),
//     Many(Vec<RcStr>),
// }
// 
// impl HeaderValue {
//     fn concat(&mut self, mut other: HeaderValue) {
//         match self {
//             HeaderValue::One(value) => {
//                 let values = match other {
//                     HeaderValue::One(other) => vec![mem::take(value), other],
//                     HeaderValue::Many(mut others) => {
//                         others.insert(0, mem::take(value));
//                         others
//                     }
//                 };
// 
//                 *self = HeaderValue::Many(values);
//             }
//             HeaderValue::Many(values) => match other {
//                 HeaderValue::One(other) => {
//                     values.push(other);
//                 }
//                 HeaderValue::Many(ref mut others) => {
//                     values.append(others);
//                 }
//             },
//         }
//     }
// }
// 
// impl<T> From<T> for HeaderValue
// where
//     RcStr: From<T>,
// {
//     fn from(value: T) -> Self {
//         HeaderValue::One(value.into())
//     }
// }
// 
// // impl From<Vec<RcStr>> for HeaderValue {
// //     fn from(values: Vec<RcStr>) -> Self {
// //     }
// 
// impl PartialEq<str> for HeaderName {
//     fn eq(&self, other: &str) -> bool {
//         self.eq_ignore_ascii_case(other)
//     }
// }
// 
// impl PartialEq<HeaderName> for str {
//     fn eq(&self, other: &HeaderName) -> bool {
//         other.eq_ignore_ascii_case(self)
//     }
// }
// 
// impl<'a> PartialEq<&'a str> for HeaderName {
//     fn eq(&self, other: &&'a str) -> bool {
//         self.eq_ignore_ascii_case(other)
//     }
// }
// 
// impl<'a> PartialEq<HeaderName> for &'a str {
//     fn eq(&self, other: &HeaderName) -> bool {
//         other.eq_ignore_ascii_case(self)
//     }
// }
// 
// impl Hash for HeaderName {
//     fn hash<H: Hasher>(&self, hasher: &mut H) {
//         for byte in self.0.bytes() {
//             hasher.write_u8(byte.to_ascii_lowercase());
//         }
//     }
// }
// 
// impl PartialOrd for HeaderName {
//     fn partial_cmp(&self, other: &HeaderName) -> Option<Ordering> {
//         Some(self.cmp(other))
//     }
// }
// 
// impl Ord for HeaderName {
//     fn cmp(&self, other: &Self) -> Ordering {
//         self.0
//             .chars()
//             .map(|c| c.to_ascii_lowercase())
//             .cmp(other.0.chars().map(|c| c.to_ascii_lowercase()))
//     }
// }
// 
// impl fmt::Display for HeaderName {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         self.0.fmt(f)
//     }
// }
