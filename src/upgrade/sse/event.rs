//! SSE event

/// SSE event
#[derive(Debug, Clone)]
pub struct Event {
    /// Event type
    pub event: Option<String>,
    /// Event data
    pub data: String,
    /// Event ID
    pub id: Option<String>,
}

impl Event {
    /// Create new event
    pub fn new(data: impl Into<String>) -> Self {
        Self {
            event: None,
            data: data.into(),
            id: None,
        }
    }

    /// Create data event
    pub fn data(data: impl Into<String>) -> Self {
        Self::new(data)
    }

    /// Set event type
    pub fn with_event(mut self, event: impl Into<String>) -> Self {
        self.event = Some(event.into());
        self
    }

    /// Set event ID
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Format as SSE
    pub fn format(&self) -> String {
        // TODO: Format as SSE protocol
        todo!("Implement Event::format")
    }
}
