/// A service type that is registered on application startup, and created per-request.
///
/// Scoped services have context about the current HTTP request, and other application state.
pub trait ScopedService: HasContext {}

impl<T> ScopedService for T where T: HasContext {}

/// A service that is created and registered once on application startup.
pub trait Singleton: 'static {}

impl<T> Singleton for T where T: 'static {}
