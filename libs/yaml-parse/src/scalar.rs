//! Scalar value parsing for YAML

use parsing::{Input, ParseResult, tag, take_while, take_while1, opt, map, pair, error_at};

/// YAML scalar value
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    String(String),
    Number(u64),
    Float(f64),
    Bool(bool),
    Null,
    List(Vec<Value>),
    Map(Vec<(String, Value)>),
}

/// Parse a quoted string (double or single quotes)
pub fn quoted_string<'a>() -> impl Fn(Input<'a>) -> ParseResult<'a, String> {
    move |input: Input<'a>| -> ParseResult<'a, String> {
        if input.starts_with('"') {
            let inner = &input[1..];
            match inner.find('"') {
                Some(pos) => {
                    let content = &inner[..pos];
                    Ok((&inner[pos + 1..], content.to_string()))
                }
                None => Err(error_at(input, "unterminated double-quoted string", 0)),
            }
        } else if input.starts_with('\'') {
            let inner = &input[1..];
            match inner.find('\'') {
                Some(pos) => {
                    let content = &inner[..pos];
                    Ok((&inner[pos + 1..], content.to_string()))
                }
                None => Err(error_at(input, "unterminated single-quoted string", 0)),
            }
        } else {
            Err(error_at(input, "expected quoted string", 0))
        }
    }
}

/// Parse an unquoted scalar (stops at :, whitespace, newline, etc.)
pub fn unquoted_scalar<'a>() -> impl Fn(Input<'a>) -> ParseResult<'a, &'a str> {
    move |input: Input<'a>| -> ParseResult<'a, &'a str> {
        let result = take_while(|c| c != ':' && c != '\n' && c != '\r' && c != '#' && c != ',' && c != ' ')(input);
        match result {
            Ok((rest, val)) => Ok((rest, val.trim())),
            Err(e) => Err(e),
        }
    }
}

/// Parse a boolean value
pub fn boolean<'a>() -> impl Fn(Input<'a>) -> ParseResult<'a, bool> {
    move |input: Input<'a>| -> ParseResult<'a, bool> {
        if input.starts_with("true") {
            Ok((&input[4..], true))
        } else if input.starts_with("false") {
            Ok((&input[5..], false))
        } else {
            Err(error_at(input, "expected boolean (true/false)", 0))
        }
    }
}

/// Parse a number (integer)
pub fn number<'a>() -> impl Fn(Input<'a>) -> ParseResult<'a, u64> {
    move |input: Input<'a>| -> ParseResult<'a, u64> {
        let result = take_while1(|c| c.is_ascii_digit());
        match result(input) {
            Ok((rest, digits)) => {
                match digits.parse::<u64>() {
                    Ok(n) => Ok((rest, n)),
                    Err(_) => Err(error_at(input, "invalid number", 0)),
                }
            }
            Err(_) => Err(error_at(input, "expected number", 0)),
        }
    }
}

/// Parse a float number
pub fn float<'a>() -> impl Fn(Input<'a>) -> ParseResult<'a, f64> {
    move |input: Input<'a>| -> ParseResult<'a, f64> {
        let integer_part = take_while1(|c| c.is_ascii_digit());
        let decimal_part = pair(tag("."), take_while(|c| c.is_ascii_digit()));
        
        let float_parser = map(
            pair(integer_part, opt(decimal_part)),
            |(int, dec)| {
                let s = if let Some((_, digits)) = dec {
                    format!("{}.{}", int, digits)
                } else {
                    int.to_string()
                };
                s.parse::<f64>().unwrap_or(0.0)
            }
        );
        
        float_parser(input)
    }
}

/// Parse a null value
pub fn null<'a>() -> impl Fn(Input<'a>) -> ParseResult<'a, ()> {
    move |input: Input<'a>| -> ParseResult<'a, ()> {
        let null_parser = map(tag("null"), |_| ());
        let tilde_parser = map(tag("~"), |_| ());
        
        match null_parser(input) {
            Ok(result) => Ok(result),
            Err(_) => tilde_parser(input),
        }
    }
}

/// Parse any scalar value
pub fn scalar<'a>() -> impl Fn(Input<'a>) -> ParseResult<'a, Value> {
    move |input: Input<'a>| -> ParseResult<'a, Value> {
        if input.starts_with('"') || input.starts_with('\'') {
            match quoted_string()(input) {
                Ok((rest, s)) => return Ok((rest, Value::String(s))),
                Err(_) => {}
            }
        }
        
        if input.starts_with("null") || input.starts_with('~') {
            match null()(input) {
                Ok((rest, ())) => return Ok((rest, Value::Null)),
                Err(_) => {}
            }
        }
        
        match boolean()(input) {
            Ok((rest, b)) => return Ok((rest, Value::Bool(b))),
            Err(_) => {}
        }
        
        if input.chars().next().map(|c| c.is_ascii_digit() || c == '.').unwrap_or(false) {
            if input.contains('.') {
                match float()(input) {
                    Ok((rest, f)) => return Ok((rest, Value::Float(f))),
                    Err(_) => {}
                }
            }
            
            match number()(input) {
                Ok((rest, n)) => return Ok((rest, Value::Number(n))),
                Err(_) => {}
            }
        }
        
        match unquoted_scalar()(input) {
            Ok((rest, s)) => {
                if s.is_empty() {
                    Err(error_at(input, "expected scalar value", 0))
                } else {
                    Ok((rest, Value::String(s.to_string())))
                }
            }
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quoted_string_double() {
        let result = quoted_string()("\"hello world\"");
        assert_eq!(result, Ok(("", "hello world".to_string())));
    }

    #[test]
    fn test_quoted_string_single() {
        let result = quoted_string()("'hello world'");
        assert_eq!(result, Ok(("", "hello world".to_string())));
    }

    #[test]
    fn test_unquoted_scalar() {
        let result = unquoted_scalar()("hello world");
        assert_eq!(result, Ok((" world", "hello")));
    }

    #[test]
    fn test_boolean_true() {
        let result = boolean()("true");
        assert_eq!(result, Ok(("", true)));
    }

    #[test]
    fn test_boolean_false() {
        let result = boolean()("false");
        assert_eq!(result, Ok(("", false)));
    }

    #[test]
    fn test_number() {
        let result = number()("123abc");
        assert_eq!(result, Ok(("abc", 123)));
    }

    #[test]
    fn test_null() {
        let result = null()("null");
        assert_eq!(result, Ok(("", ())));
    }

    #[test]
    fn test_null_tilde() {
        let result = null()("~");
        assert_eq!(result, Ok(("", ())));
    }

    #[test]
    fn test_scalar_string() {
        let result = scalar()("hello");
        assert_eq!(result, Ok(("", Value::String("hello".to_string()))));
    }

    #[test]
    fn test_scalar_number() {
        let result = scalar()("42");
        assert_eq!(result, Ok(("", Value::Number(42))));
    }

    #[test]
    fn test_scalar_bool() {
        let result = scalar()("true");
        assert_eq!(result, Ok(("", Value::Bool(true))));
    }

    #[test]
    fn test_scalar_null() {
        let result = scalar()("null");
        assert_eq!(result, Ok(("", Value::Null)));
    }
}

    #[test]
    fn test_float() {
        let result = float()("3.14");
        assert_eq!(result, Ok(("", 3.14)));
    }

    #[test]
    fn test_scalar_float() {
        let result = scalar()("3.14");
        assert_eq!(result, Ok(("", Value::Float(3.14))));
    }

    #[test]
    fn test_quoted_string_escaped() {
        let result = quoted_string()("\"hello\"world\"");
        assert_eq!(result, Ok(("", "hello\"world".to_string())));
    }

    #[test]
    fn test_unquoted_with_space() {
        let result = unquoted_scalar()("hello world more");
        assert_eq!(result, Ok((" world more", "hello")));
    }
