//! App builder

use crate::app::App;

/// App builder
pub struct AppBuilder<E = ()> {
    env: E,
}

impl AppBuilder<()> {
    /// Create new builder
    pub fn new() -> Self {
        Self { env: () }
    }
}

impl<E> AppBuilder<E>
where
    E: Send + Sync + 'static,
{
    /// Set environment
    pub fn with_env<T>(self, env: T) -> AppBuilder<T>
    where
        T: Send + Sync + 'static,
    {
        AppBuilder { env }
    }

    /// Build app
    pub fn build(self) -> App<E> {
        App::with_env(self.env)
    }
}

impl Default for AppBuilder<()> {
    fn default() -> Self {
        Self::new()
    }
}
