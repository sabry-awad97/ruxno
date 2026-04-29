//! Environment/DI container

/// Environment trait marker
///
/// This trait is automatically implemented for all types that are Send + Sync + 'static.
/// It serves as a marker trait for dependency injection.
pub trait Environment: Send + Sync + 'static {}

// Blanket implementation for all types that meet the bounds
impl<T: Send + Sync + 'static> Environment for T {}
