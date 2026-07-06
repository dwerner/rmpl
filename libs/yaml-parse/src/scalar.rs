//! Scalar value parsing for YAML

use parsing::{Input, ParseResult, tag, take_while, opt, error_at, map};

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
pub fn quoted_string() -> impl Fn(Input) -> ParseResult<'_, String> {
    move |input: Input| -> ParseResult<'_, String> {
        // Try double quotes
        if input.starts_with('"') {
            let inner = &input[1..];
            match inner.find('"') {
                Some(pos) => {
                    let content = &inner[..pos];
                    Ok((&inner[pos + 1..], content.to_string()))
                }
                None => Err(error_at(input, "unterminated double-quoted string", 0)),
            }
        }
        // Try single quotes
        else if input.starts_with('\'') {
            let inner = &input[1..];
            match inner.find('\'') {
                Some(pos) => {
                    let content = &inner[..pos];
                    Ok((&inner[pos + 1..], content.to_string()))
                }
                None => Err(error_at(input, "unterminated single-quoted string", 0)),
            }
        }
        else {
            Err(error_at(input, "expected quoted string", 0))
        }
    }
}

/// Parse an unquoted scalar (stops at : whitespace, or newline)
pub fn unquoted_scalar() -> impl Fn(Input) -> ParseResult<'_, &str> {
    move |input: Input| -> ParseResult<'_, &str> {
        let result = take_while(|c| c != ':' && c != '\n' && c != '\r' && c != '#' && c != ',' && c != ' ')(input);
        match result {
            Ok((rest, val)) => Ok((rest, val.trim())),
            Err(e) => Err(e),
        }
    }
}

/// Parse a boolean value
pub fn boolean() -> impl Fn(Input) -> ParseResult<'_, bool> {
    move |input: Input| -> ParseResult<'_, bool> {
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
pub fn number() -> impl Fn(Input) -> ParseResult<'_, u64> {
    move |input: Input| -> ParseResult<'_, u64> {
        let result = take_while(|c| c.is_ascii_digit());
        match result(input) {
            Ok((rest, digits)) if !digits.is_empty() => {
                match digits.parse::<u64>() {
                    Ok(n) => Ok((rest, n)),
                    Err(_) => Err(error_at(input, "invalid number", 0)),
                }
            }
            Ok((_rest, _)) => Err(error_at(input, "expected number", 0)),
            Err(e) => Err(e),
        }
    }
}

/// Parse a float number
pub fn float() -> impl Fn(Input) -> ParseResult<'_, f64> {
    move |input: Input| -> ParseResult<'_, f64> {
        let integer_part = take_while(|c| c.is_ascii_digit());
        let decimal_part = parsing::pair(tag("."), take_while(|c| c.is_ascii_digit()));
        
        let float_parser = map(
            parsing::pair(integer_part, opt(decimal_part)),
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
pub fn null() -> impl Fn(Input) -> ParseResult<'_, ()> {
    move |input: Input| -> ParseResult<'_, ()> {
        let null_parser = map(tag("null"), |_| ());
        let tilde_parser = map(tag("~"), |_| ());
        
        match null_parser(input) {
            Ok(result) => Ok(result),
            Err(_) => tilde_parser(input),
        }
    }
}

/// Parse any scalar value
pub fn scalar() -> impl Fn(Input) -> ParseResult<'_, Value> {
    move |input: Input| -> ParseResult<'_, Value> {
        // Try quoted string first
        if input.starts_with('"') || input.starts_with('\'') {
            match quoted_string()(input) {
                Ok((rest, s)) => return Ok((rest, Value::String(s))),
                Err(_) => {}
            }
        }
        
        // Try null
        if input.starts_with("null") || input.starts_with('~') {
            match null()(input) {
                Ok((rest, ())) => return Ok((rest, Value::Null)),
                Err(_) => {}
            }
        }
        
        // Try boolean
        match boolean()(input) {
            Ok((rest, b)) => return Ok((rest, Value::Bool(b))),
            Err(_) => {}
        }
        
        // Try float (must check before integer to handle decimals)
        if input.chars().next().map(|c| c.is_ascii_digit() || c == '.').unwrap_or(false) {
            if input.contains('.') {
                match float()(input) {
                    Ok((rest, f)) => return Ok((rest, Value::Float(f))),
                    Err(_) => {}
                }
            }
            
            // Try integer
            match number()(input) {
                Ok((rest, n)) => return Ok((rest, Value::Number(n))),
                Err(_) => {}
            }
        }
        
        // Fall back to unquoted string
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
