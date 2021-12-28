/// An optional extractor argument.
#[derive(Clone)]
pub struct OptionalArgument<T> {
    /// The argument value.
    pub value: Option<T>,
    /// The name of the struct field being extracted.
    pub field_name: &'static str,
}

#[derive(Clone)]
pub struct RequiredArgument<T> {
    /// The argument value.
    pub value: T,
    /// The name of the struct field being extracted.
    pub field_name: &'static str,
}

impl<T> RequiredArgument<T> {
    #[doc(hidden)]
    pub fn new(field_name: &'static str, value: T) -> Self {
        Self { field_name, value }
    }
}

/// A placeholder for no extractor argument.
#[derive(Clone)]
pub struct NoArgument {
    /// The name of the struct field being extracted.
    pub field_name: &'static str,
}

impl NoArgument {
    #[doc(hidden)]
    pub fn new(field_name: &'static str) -> Self {
        Self { field_name }
    }
}

// Used by the macro.
impl<T> From<RequiredArgument<T>> for OptionalArgument<T> {
    fn from(arg: RequiredArgument<T>) -> Self {
        Self {
            value: Some(arg.value),
            field_name: arg.field_name,
        }
    }
}

impl<T> From<NoArgument> for OptionalArgument<T> {
    fn from(arg: NoArgument) -> Self {
        Self {
            value: None,
            field_name: arg.field_name,
        }
    }
}
