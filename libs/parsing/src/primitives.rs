//! Basic parser primitives

use crate::input::{error_at, Input, ParseResult};

/// Parse a literal string tag
pub fn tag<'a>(s: &'a str) -> impl Fn(Input<'a>) -> ParseResult<'a, &'a str> + 'a {
    move |input: Input<'a>| -> ParseResult<'a, &'a str> {
        if input.starts_with(s) {
            Ok((&input[s.len()..], &input[..s.len()]))
        } else {
            Err(error_at(input, &format!("expected tag '{}'", s), 0))
        }
    }
}

/// Parse one of the given characters
pub fn one_of<'a>(chars: &'a str) -> impl Fn(Input<'a>) -> ParseResult<'a, char> + 'a {
    move |input: Input<'a>| -> ParseResult<'a, char> {
        if let Some(c) = input.chars().next() {
            if chars.contains(c) {
                let rest = &input[c.len_utf8()..];
                return Ok((rest, c));
            }
        }
        Err(error_at(input, &format!("expected one of '{}'", chars), 0))
    }
}

/// Parse zero or more characters matching the predicate
pub fn take_while<F>(pred: F) -> impl Fn(Input) -> ParseResult<'_, &str>
where
    F: Fn(char) -> bool + 'static,
{
    move |input: Input| -> ParseResult<'_, &str> {
        let len = input
            .char_indices()
            .take_while(|(_, c)| pred(*c))
            .last()
            .map(|(i, _)| i + char_len(input, i))
            .unwrap_or(0);
        Ok((&input[len..], &input[..len]))
    }
}

/// Parse one or more characters matching the predicate
pub fn take_while1<F>(pred: F) -> impl Fn(Input) -> ParseResult<'_, &str>
where
    F: Fn(char) -> bool + 'static,
{
    move |input: Input| -> ParseResult<'_, &str> {
        let len = input
            .char_indices()
            .take_while(|(_, c)| pred(*c))
            .last()
            .map(|(i, _)| i + char_len(input, i))
            .unwrap_or(0);

        if len == 0 {
            Err(error_at(input, "expected at least one matching character", 0))
        } else {
            Ok((&input[len..], &input[..len]))
        }
    }
}

/// Parse characters until the delimiter is found
pub fn take_until<'a>(delim: &'a str) -> impl Fn(Input<'a>) -> ParseResult<'a, &'a str> + 'a {
    move |input: Input<'a>| -> ParseResult<'a, &'a str> {
        match input.find(delim) {
            Some(pos) => Ok((&input[pos..], &input[..pos])),
            None => Err(error_at(input, &format!("expected '{}' not found", delim), 0)),
        }
    }
}

/// Parse exactly n characters
pub fn take(n: usize) -> impl Fn(Input) -> ParseResult<'_, &str> {
    move |input: Input| -> ParseResult<'_, &str> {
        let len = input
            .char_indices()
            .nth(n)
            .map(|(i, _)| i)
            .unwrap_or_else(|| input.len());

        if len < n && input.len() < n {
            Err(error_at(input, &format!("expected {} characters", n), 0))
        } else {
            Ok((&input[len..], &input[..len]))
        }
    }
}

/// Parse end of input
pub fn eof() -> impl Fn(Input) -> ParseResult<'_, ()> {
    move |input: Input| -> ParseResult<'_, ()> {
        if input.is_empty() {
            Ok((input, ()))
        } else {
            Err(error_at(input, "expected end of input", 0))
        }
    }
}

/// Parse a newline
pub fn newline() -> impl Fn(Input) -> ParseResult<'_, &str> {
    move |input: Input| -> ParseResult<'_, &str> {
        if input.starts_with('\n') {
            Ok((&input[1..], "\n"))
        } else {
            Err(error_at(input, "expected newline", 0))
        }
    }
}

/// Parse a comment starting with # until end of line
pub fn comment() -> impl Fn(Input) -> ParseResult<'_, &str> {
    move |input: Input| -> ParseResult<'_, &str> {
        if !input.starts_with('#') {
            return Err(error_at(input, "expected comment starting with #", 0));
        }
        match input.find('\n') {
            Some(pos) => Ok((&input[pos..], &input[..pos])),
            None => Ok(("", input)),
        }
    }
}

// Helper to get char length at position
fn char_len(s: &str, index: usize) -> usize {
    s[index..].chars().next().map(|c| c.len_utf8()).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tag_success() {
        let result = tag("hello")("hello world");
        assert_eq!(result, Ok((" world", "hello")));
    }

    #[test]
    fn test_tag_failure() {
        let result = tag("hello")("goodbye world");
        assert!(result.is_err());
    }

    #[test]
    fn test_one_of_success() {
        let result = one_of("aeiou")("apple");
        assert_eq!(result, Ok(("pple", 'a')));
    }

    #[test]
    fn test_one_of_failure() {
        let result = one_of("aeiou")("xyz");
        assert!(result.is_err());
    }

    #[test]
    fn test_take_while_success() {
        let result = take_while(|c| c.is_alphabetic())("hello123");
        assert_eq!(result, Ok(("123", "hello")));
    }

    #[test]
    fn test_take_while_empty() {
        let result = take_while(|c| c.is_alphabetic())("123hello");
        assert_eq!(result, Ok(("123hello", "")));
    }

    #[test]
    fn test_take_while1_success() {
        let result = take_while1(|c| c.is_alphabetic())("hello123");
        assert_eq!(result, Ok(("123", "hello")));
    }

    #[test]
    fn test_take_while1_failure() {
        let result = take_while1(|c| c.is_alphabetic())("123hello");
        assert!(result.is_err());
    }

    #[test]
    fn test_take_until_success() {
        let result = take_until(":")("key: value");
        assert_eq!(result, Ok((": value", "key")));
    }

    #[test]
    fn test_take_until_failure() {
        let result = take_until(":")("no colon here");
        assert!(result.is_err());
    }

    #[test]
    fn test_take_success() {
        let result = take(5)("hello world");
        assert_eq!(result, Ok((" world", "hello")));
    }

    #[test]
    fn test_eof_success() {
        let result = eof()("");
        assert_eq!(result, Ok(("", ())));
    }

    #[test]
    fn test_eof_failure() {
        let result = eof()("not empty");
        assert!(result.is_err());
    }

    #[test]
    fn test_newline() {
        let result = newline()("\nhello");
        assert_eq!(result, Ok(("hello", "\n")));
    }

    #[test]
    fn test_comment() {
        let result = comment()("# this is a comment\nmore");
        assert_eq!(result, Ok(("\nmore", "# this is a comment")));
    }
}
