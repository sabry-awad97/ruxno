//! Route matching result

use crate::core::BoxedHandler;
use crate::routing::Params;

/// Result of route matching
pub struct Match<E = ()> {
    /// Matched handler
    pub handler: BoxedHandler<E>,

    /// Extracted path parameters
    pub params: Params,
}

impl<E> Match<E> {
    /// Create a new match result
    pub fn new(handler: BoxedHandler<E>, params: Params) -> Self {
        Self { handler, params }
    }
}
