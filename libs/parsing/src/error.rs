//! Parse error types with position tracking

/// Error that occurs during parsing
#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    pub message: String,
    pub line: usize,    // 1-indexed
    pub column: usize,  // 1-indexed
}

impl ParseError {
    /// Create a new parse error at the current position
    pub fn new(message: &str, line: usize, column: usize) -> Self {
        Self {
            message: message.to_string(),
            line,
            column,
        }
    }

    /// Create an error with a custom message
    pub fn with_message(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            line: 1,
            column: 1,
        }
    }

    /// Format the error with position information
    pub fn format(&self) -> String {
        format!("{} at line {}, column {}", self.message, self.line, self.column)
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.format())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_error_new() {
        let err = ParseError::new("unexpected end of input", 5, 10);
        assert_eq!(err.message, "unexpected end of input");
        assert_eq!(err.line, 5);
        assert_eq!(err.column, 10);
    }

    #[test]
    fn test_parse_error_format() {
        let err = ParseError::new("expected tag", 3, 5);
        assert_eq!(err.format(), "expected tag at line 3, column 5");
    }

    #[test]
    fn test_parse_error_display() {
        let err = ParseError::new("failed", 1, 1);
        assert_eq!(err.to_string(), "failed at line 1, column 1");
    }
}
