//! Path pattern parsing and validation

/// Pattern error
#[derive(Debug, Clone)]
pub enum PatternError {
    /// Empty pattern
    EmptyPattern,
    /// Invalid syntax
    InvalidSyntax(String),
}

impl std::fmt::Display for PatternError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PatternError::EmptyPattern => write!(f, "Empty pattern"),
            PatternError::InvalidSyntax(msg) => write!(f, "Invalid syntax: {}", msg),
        }
    }
}

impl std::error::Error for PatternError {}

/// Path pattern
#[derive(Debug, Clone)]
pub struct Pattern {
    /// Original pattern string
    original: String,
    /// Matchit-compatible pattern
    matchit_pattern: String,
}

impl Pattern {
    /// Parse a pattern
    pub fn parse(pattern: &str) -> Result<Self, PatternError> {
        if pattern.is_empty() {
            return Err(PatternError::EmptyPattern);
        }

        // TODO: Implement pattern parsing
        // - Convert :param to {param}
        // - Validate syntax
        // - Handle wildcards
        todo!("Implement Pattern::parse")
    }

    /// Get original pattern
    pub fn original(&self) -> &str {
        &self.original
    }

    /// Get matchit-compatible pattern
    pub fn matchit_pattern(&self) -> &str {
        &self.matchit_pattern
    }
}
