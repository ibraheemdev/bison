use crate::{handler, http};

use std::collections::HashMap;

use matchit::Node;

pub struct Router<W> {
    wrap: W,
    routes: HashMap<http::Method, Node<handler::Erased>>,
}
