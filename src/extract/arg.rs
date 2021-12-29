//! Arguments that can be passed to extractors.
//!
//! ...

/// An extractor argument.
///
/// See the [module level documentation](super) for details.
pub trait Argument<T> {
    /// Create the argument.
    fn new(field_name: &'static str, value: T) -> Self;
}

impl<T> Argument<T> for T {
    fn new(_: &'static str, value: T) -> Self {
        value
    }
}

/// An extractor argument with a default value.
///
/// See the [module level documentation](super) for details.
pub trait DefaultArgument {
    fn new(field_name: &'static str) -> Self;
}

impl DefaultArgument for () {
    fn new(_: &'static str) -> Self {}
}

impl<T> Argument<T> for Option<T> {
    fn new(_: &'static str, value: T) -> Self {
        Some(value)
    }
}

impl<T> DefaultArgument for Option<T> {
    fn new(_: &'static str) -> Self {
        None
    }
}

/// An argument representing the name of the field being extracted.
///
/// See the [module level documentation](super) for details.
pub struct FieldName(&'static str);

impl FieldName {
    /// Returns the name of the field as a string.
    pub fn as_str(&self) -> &'static str {
        self.0
    }
}

impl DefaultArgument for FieldName {
    fn new(field_name: &'static str) -> Self {
        FieldName(field_name)
    }
}

/// The name of the parameter to be extracted by
/// [`extract::path`](super::path()) or [`extract::query`](super::query()).
pub struct ParamName(pub &'static str);

impl Argument<&'static str> for ParamName {
    fn new(_: &'static str, value: &'static str) -> Self {
        ParamName(value)
    }
}

impl DefaultArgument for ParamName {
    fn new(field_name: &'static str) -> Self {
        ParamName(field_name)
    }
}
