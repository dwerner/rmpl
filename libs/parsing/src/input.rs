//! Input type and position tracking

use crate::error::ParseError;

/// Input type for parsers (borrowed string)
pub type Input<'a> = &'a str;

/// Result type for parsers
/// Returns (remaining_input, parsed_value) on success, or ParseError on failure
pub type ParseResult<'a, T> = Result<(Input<'a>, T), ParseError>;

/// Calculate line and column from byte index
/// Line and column are 1-indexed
pub fn get_position(input: Input, index: usize) -> (usize, usize) {
    let prefix = &input[..index.min(input.len())];
    let line = prefix.matches('\n').count() + 1;
    let last_newline = prefix.rfind('\n').map(|i| i + 1).unwrap_or(0);
    let column = index - last_newline + 1;
    (line, column)
}

/// Create a parse error at a specific position in the input
pub fn error_at(input: Input, message: &str, index: usize) -> ParseError {
    let (line, column) = get_position(input, index);
    ParseError::new(message, line, column)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_position_start() {
        let input = "hello world";
        let (line, column) = get_position(input, 0);
        assert_eq!(line, 1);
        assert_eq!(column, 1);
    }

    #[test]
    fn test_get_position_middle() {
        let input = "hello world";
        let (line, column) = get_position(input, 6);
        assert_eq!(line, 1);
        assert_eq!(column, 7);
    }

    #[test]
    fn test_get_position_multiline() {
        let input = "hello\nworld";
        let (line, column) = get_position(input, 6); // 'w' in world
        assert_eq!(line, 2);
        assert_eq!(column, 1);
    }

    #[test]
    fn test_get_position_after_newline() {
        let input = "line1\nline2\nline3";
        let (line, column) = get_position(input, 13); // 'i' in line3
        assert_eq!(line, 3);
        assert_eq!(column, 2);
    }

    #[test]
    fn test_error_at() {
        let input = "hello\nworld";
        let err = error_at(input, "unexpected character", 6);
        assert_eq!(err.line, 2);
        assert_eq!(err.column, 1);
        assert_eq!(err.message, "unexpected character");
    }
}
