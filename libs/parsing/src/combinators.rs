//! Parser combinators for composing parsers

use crate::input::{Input, ParseResult};
use crate::error::ParseError;

/// Try multiple parsers, return the first successful one
pub fn alt<'a, 'b, T>(parsers: &'b [impl Fn(Input<'a>) -> ParseResult<'a, T> + 'b]) -> impl Fn(Input<'a>) -> ParseResult<'a, T> + 'b {
    move |input: Input<'a>| -> ParseResult<'a, T> {
        for parser in parsers {
            if let Ok(result) = parser(input) {
                return Ok(result);
            }
        }
        Err(ParseError::new("no alternative parser succeeded", 1, 1))
    }
}

/// Make a parser optional
pub fn opt<'a, T>(parser: impl Fn(Input<'a>) -> ParseResult<'a, T>) -> impl Fn(Input<'a>) -> ParseResult<'a, Option<T>> {
    move |input: Input<'a>| -> ParseResult<'a, Option<T>> {
        match parser(input) {
            Ok((rest, value)) => Ok((rest, Some(value))),
            Err(_) => Ok((input, None)),
        }
    }
}

/// Parse zero or more occurrences
pub fn many<'a, T>(parser: impl Fn(Input<'a>) -> ParseResult<'a, T>) -> impl Fn(Input<'a>) -> ParseResult<'a, Vec<T>> {
    move |mut input: Input<'a>| -> ParseResult<'a, Vec<T>> {
        let mut results = Vec::new();

        loop {
            match parser(input) {
                Ok((rest, value)) => {
                    results.push(value);
                    input = rest;
                }
                Err(_) => break,
            }
        }

        Ok((input, results))
    }
}

/// Parse one or more occurrences
pub fn many1<'a, T>(parser: impl Fn(Input<'a>) -> ParseResult<'a, T>) -> impl Fn(Input<'a>) -> ParseResult<'a, Vec<T>> {
    move |mut input: Input<'a>| -> ParseResult<'a, Vec<T>> {
        let mut results = Vec::new();

        // First iteration is required
        match parser(input) {
            Ok((rest, value)) => {
                results.push(value);
                input = rest;
            }
            Err(e) => return Err(e),
        }

        // Continue parsing more
        loop {
            match parser(input) {
                Ok((rest, value)) => {
                    results.push(value);
                    input = rest;
                }
                Err(_) => break,
            }
        }

        Ok((input, results))
    }
}

/// Parse two parsers in sequence, return both results as a tuple
pub fn pair<'a, T1, T2>(
    p1: impl Fn(Input<'a>) -> ParseResult<'a, T1>,
    p2: impl Fn(Input<'a>) -> ParseResult<'a, T2>,
) -> impl Fn(Input<'a>) -> ParseResult<'a, (T1, T2)> {
    move |input: Input<'a>| -> ParseResult<'a, (T1, T2)> {
        let (rest, t1) = p1(input)?;
        let (rest, t2) = p2(rest)?;
        Ok((rest, (t1, t2)))
    }
}

/// Parse three parsers in sequence
pub fn tuple3<'a, T1, T2, T3>(
    p1: impl Fn(Input<'a>) -> ParseResult<'a, T1>,
    p2: impl Fn(Input<'a>) -> ParseResult<'a, T2>,
    p3: impl Fn(Input<'a>) -> ParseResult<'a, T3>,
) -> impl Fn(Input<'a>) -> ParseResult<'a, (T1, T2, T3)> {
    move |input: Input<'a>| -> ParseResult<'a, (T1, T2, T3)> {
        let (rest, t1) = p1(input)?;
        let (rest, t2) = p2(rest)?;
        let (rest, t3) = p3(rest)?;
        Ok((rest, (t1, t2, t3)))
    }
}

/// Parse with a separator between items
pub fn separated_list<'a, T, D>(
    sep: impl Fn(Input<'a>) -> ParseResult<'a, D>,
    parser: impl Fn(Input<'a>) -> ParseResult<'a, T>,
) -> impl Fn(Input<'a>) -> ParseResult<'a, Vec<T>> {
    move |input: Input<'a>| -> ParseResult<'a, Vec<T>> {
        let mut results = Vec::new();

        // Parse first item
        match parser(input) {
            Ok((mut rest, value)) => {
                results.push(value);

                // Parse separator + item repeatedly
                loop {
                    match sep(rest) {
                        Ok((after_sep, _)) => {
                            match parser(after_sep) {
                                Ok((new_rest, value)) => {
                                    results.push(value);
                                    rest = new_rest;
                                }
                                Err(_) => break,
                            }
                        }
                        Err(_) => break,
                    }
                }

                Ok((rest, results))
            }
            Err(_) => Ok((input, results)),
        }
    }
}

/// Transform the output of a parser
pub fn map<'a, I, O>(
    parser: impl Fn(Input<'a>) -> ParseResult<'a, I>,
    f: impl Fn(I) -> O,
) -> impl Fn(Input<'a>) -> ParseResult<'a, O> {
    move |input: Input<'a>| -> ParseResult<'a, O> {
        let (rest, value) = parser(input)?;
        Ok((rest, f(value)))
    }
}

/// Ignore the output of a parser, return a constant value
pub fn value<'a, T, I>(
    val: T,
    parser: impl Fn(Input<'a>) -> ParseResult<'a, I>,
) -> impl Fn(Input<'a>) -> ParseResult<'a, T>
where
    T: Clone,
{
    move |input: Input<'a>| -> ParseResult<'a, T> {
        parser(input).map(|(rest, _)| (rest, val.clone()))
    }
}

/// Consume the parser but discard the result
pub fn ignore<'a, I>(parser: impl Fn(Input<'a>) -> ParseResult<'a, I>) -> impl Fn(Input<'a>) -> ParseResult<'a, ()> {
    move |input: Input<'a>| -> ParseResult<'a, ()> {
        parser(input).map(|(rest, _)| (rest, ()))
    }
}

/// Prepend context to error messages
pub fn context<'a, T>(
    parser: impl Fn(Input<'a>) -> ParseResult<'a, T>,
    msg: &'static str,
) -> impl Fn(Input<'a>) -> ParseResult<'a, T> {
    move |input: Input<'a>| -> ParseResult<'a, T> {
        parser(input).map_err(|_| ParseError::new(msg, 1, 1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::{tag, take_while};

    #[test]
    fn test_opt_success() {
        let parser = opt(tag("hello"));
        let result = parser("hello world");
        assert_eq!(result, Ok((" world", Some("hello"))));
    }

    #[test]
    fn test_opt_failure() {
        let parser = opt(tag("hello"));
        let result = parser("goodbye world");
        assert_eq!(result, Ok(("goodbye world", None)));
    }

    #[test]
    fn test_many_zero() {
        let parser = many(tag("a"));
        let result = parser("bbb");
        assert_eq!(result, Ok(("bbb", vec![])));
    }

    #[test]
    fn test_many_multiple() {
        let parser = many(tag("a"));
        let result = parser("aaabbb");
        assert_eq!(result, Ok(("bbb", vec!["a", "a", "a"])));
    }

    #[test]
    fn test_many1_success() {
        let parser = many1(tag("a"));
        let result = parser("aaabbb");
        assert_eq!(result, Ok(("bbb", vec!["a", "a", "a"])));
    }

    #[test]
    fn test_many1_failure() {
        let parser = many1(tag("a"));
        let result = parser("bbb");
        assert!(result.is_err());
    }

    #[test]
    fn test_pair() {
        let parser = pair(tag("hello"), tag(" world"));
        let result = parser("hello world");
        assert_eq!(result, Ok(("", ("hello", " world"))));
    }

    #[test]
    fn test_map() {
        let parser = map(take_while(|c| c.is_alphabetic()), |s: &str| s.to_uppercase());
        let result = parser("Hello123");
        assert_eq!(result, Ok(("123", String::from("HELLO"))));
    }

    #[test]
    fn test_value() {
        let parser = value(42, tag("hello"));
        let result = parser("hello world");
        assert_eq!(result, Ok((" world", 42)));
    }

    #[test]
    fn test_ignore() {
        let parser = ignore(tag("hello"));
        let result = parser("hello world");
        assert_eq!(result, Ok((" world", ())));
    }

    #[test]
    fn test_alt_success_first() {
        let p1 = tag("hello");
        let p2 = tag("world");
        let parsers = [p1, p2];
        let parser = alt(&parsers);
        let result = parser("hello world");
        assert_eq!(result, Ok((" world", "hello")));
    }

    #[test]
    fn test_alt_success_second() {
        let p1 = tag("hello");
        let p2 = tag("world");
        let parsers = [p1, p2];
        let parser = alt(&parsers);
        let result = parser("world hello");
        assert_eq!(result, Ok((" hello", "world")));
    }

    #[test]
    fn test_alt_failure() {
        let p1 = tag("hello");
        let p2 = tag("world");
        let parsers = [p1, p2];
        let parser = alt(&parsers);
        let result = parser("foo bar");
        assert!(result.is_err());
    }

    #[test]
    fn test_tuple3() {
        let parser = tuple3(tag("a"), tag("b"), tag("c"));
        let result = parser("abc");
        assert_eq!(result, Ok(("", ("a", "b", "c"))));
    }

    #[test]
    fn test_separated_list() {
        let parser = separated_list(tag(","), take_while(|c| c != ','));
        let result = parser("a,b,c");
        assert_eq!(result, Ok(("", vec!["a", "b", "c"])));
    }
}
