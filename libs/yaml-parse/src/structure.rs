//! Structure parsing for YAML (key-value, lists, blocks)

use crate::scalar::{scalar, Value};
use parsing::{Input, ParseResult, ParseError, error_at};

/// Parse whitespace (spaces and tabs, not newlines)
pub fn indent() -> impl Fn(Input) -> ParseResult<'_, usize> {
    move |input: Input| -> ParseResult<'_, usize> {
        let count = input.chars().take_while(|c| *c == ' ' || *c == '\t').count();
        Ok((&input[count..], count))
    }
}

/// Parse inline whitespace
pub fn ws() -> impl Fn(Input) -> ParseResult<'_, &str> {
    move |input: Input| -> ParseResult<'_, &str> {
        let count = input.chars().take_while(|c| *c == ' ' || *c == '\t').count();
        Ok((&input[count..], &input[..count]))
    }
}

/// Parse a comment (from # to end of line)
pub fn comment() -> impl Fn(Input) -> ParseResult<'_, ()> {
    move |input: Input| -> ParseResult<'_, ()> {
        if !input.starts_with('#') {
            return Ok((input, ()));
        }
        match input.find('\n') {
            Some(pos) => Ok((&input[pos..], ())),
            None => Ok(("", ())),
        }
    }
}

/// Parse key-value pair: `key: value`
pub fn key_value() -> impl Fn(Input) -> ParseResult<'_, (String, Value)> {
    move |input: Input| -> ParseResult<'_, (String, Value)> {
        // Parse key (unquoted identifier)
        let key_end = input.find(':').ok_or_else(|| error_at(input, "expected ':' in key-value pair", 0))?;
        let key = input[..key_end].trim().to_string();
        let rest = &input[key_end..];
        
        // Parse colon and optional space
        let after_colon = if rest.starts_with(":") {
            let rest = &rest[1..];
            rest.trim_start()
        } else {
            return Err(error_at(input, "expected ':' after key", 0));
        };
        
        // Check if value is on next line (nested block)
        if after_colon.starts_with('\n') || after_colon.is_empty() {
            // Value is a nested block - need to handle this differently
            // For now, return empty map as placeholder
            Ok((after_colon, (key, Value::Map(vec![]))))
        } else {
            // Parse value on same line
            let (rest_after_val, value) = scalar()(after_colon)?;
            Ok((rest_after_val, (key, value)))
        }
    }
}

/// Parse a list item: `- value`
pub fn list_item() -> impl Fn(Input) -> ParseResult<'_, Value> {
    move |input: Input| -> ParseResult<'_, Value> {
        if !input.starts_with('-') {
            return Err(error_at(input, "expected list item starting with '-'", 0));
        }
        
        let after_dash = &input[1..];
        let after_space = after_dash.trim_start();
        
        // Parse the value
        scalar()(after_space)
    }
}

/// Parse a list: multiple `- value` items
pub fn list() -> impl Fn(Input) -> ParseResult<'_, Vec<Value>> {
    move |mut input: Input| -> ParseResult<'_, Vec<Value>> {
        let mut items = Vec::new();
        
        while input.starts_with('-') {
            let (rest, value) = list_item()(input)?;
            items.push(value);
            input = rest.trim_start_matches('\n').trim_start();
        }
        
        Ok((input, items))
    }
}

/// Parse a block (key-value pairs at same indentation level)
pub fn block() -> impl Fn(Input) -> ParseResult<'_, Vec<(String, Value)>> {
    move |input: Input| -> ParseResult<'_, Vec<(String, Value)>> {
        let mut pairs = Vec::new();
        let mut remaining = input;
        
        while let Some(colon_pos) = remaining.find(':') {
            // Check if this is actually a key-value (not inside a string)
            let before_colon = &remaining[..colon_pos];
            if before_colon.trim().is_empty() || before_colon.trim().starts_with('#') {
                break;
            }
            
            match key_value()(remaining) {
                Ok((rest, (key, value))) => {
                    pairs.push((key, value));
                    remaining = rest;
                }
                Err(_) => break,
            }
        }
        
        Ok((remaining, pairs))
    }
}

/// Parse a complete YAML document
pub fn document() -> impl Fn(Input) -> ParseResult<'_, Value> {
    move |input: Input| -> ParseResult<'_, Value> {
        // Skip leading whitespace and comments
        let remaining = input.trim_start();
        
        // Check if it's a list or a map
        if remaining.starts_with('-') {
            let (rest, items) = list()(remaining)?;
            Ok((rest, Value::List(items)))
        } else {
            let (rest, pairs) = block()(remaining)?;
            Ok((rest, Value::Map(pairs)))
        }
    }
}

/// Parse a YAML file content into a Value
pub fn parse(input: &str) -> Result<Value, ParseError> {
    match document()(input) {
        Ok((_, value)) => Ok(value),
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_indent_spaces() {
        let result = indent()("  hello");
        assert_eq!(result, Ok(("hello", 2)));
    }

    #[test]
    fn test_indent_tabs() {
        let result = indent()("\t\thello");
        assert_eq!(result, Ok(("hello", 2)));
    }

    #[test]
    fn test_indent_none() {
        let result = indent()("hello");
        assert_eq!(result, Ok(("hello", 0)));
    }

    #[test]
    fn test_key_value_simple() {
        let result = key_value()("name: value");
        assert_eq!(result, Ok(("", ("name".to_string(), Value::String("value".to_string())))));
    }

    #[test]
    fn test_key_value_number() {
        let result = key_value()("count: 42");
        assert_eq!(result, Ok(("", ("count".to_string(), Value::Number(42)))));
    }

    #[test]
    fn test_key_value_bool() {
        let result = key_value()("enabled: true");
        assert_eq!(result, Ok(("", ("enabled".to_string(), Value::Bool(true)))));
    }

    #[test]
    fn test_list_item() {
        let result = list_item()("- item");
        assert_eq!(result, Ok(("", Value::String("item".to_string()))));
    }

    #[test]
    fn test_list() {
        let result = list()("- a\n- b\n- c");
        assert_eq!(result, Ok(("", vec![
            Value::String("a".to_string()),
            Value::String("b".to_string()),
            Value::String("c".to_string()),
        ])));
    }

    #[test]
    fn test_block() {
        let result = block()("name: test\nversion: 1");
        assert!(result.is_ok());
        let (rest, pairs) = result.unwrap();
        assert_eq!(rest.trim(), "");
        assert_eq!(pairs.len(), 2);
    }

    #[test]
    fn test_parse_simple() {
        let result = parse("name: test");
        assert!(result.is_ok());
    }
}
