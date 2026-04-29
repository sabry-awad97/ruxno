//! Hyper service implementation

use crate::app::App;
use crate::http::{from_hyper_request, to_hyper_response};
use hyper::body::Incoming;
use std::sync::Arc;

/// Ruxno service for Hyper
pub struct RuxnoService<E = ()> {
    app: Arc<App<E>>,
}

impl<E> RuxnoService<E>
where
    E: Send + Sync + 'static,
{
    /// Create new service
    pub fn new(app: Arc<App<E>>) -> Self {
        Self { app }
    }

    /// Handle request
    pub async fn handle(
        &self,
        req: hyper::Request<Incoming>,
    ) -> Result<hyper::Response<http_body_util::Full<bytes::Bytes>>, std::convert::Infallible> {
        // TODO: Convert Hyper request to domain request
        // TODO: Dispatch through app
        // TODO: Convert domain response to Hyper response
        todo!("Implement RuxnoService::handle")
    }
}
