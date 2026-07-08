//! Structure parsing for YAML (key-value, lists, blocks)

use crate::scalar::{scalar, Value};
use parsing::{Input, ParseResult, error_at};

/// Parse whitespace (spaces and tabs, not newlines)
pub fn indent<'a>() -> impl Fn(Input<'a>) -> ParseResult<'a, usize> {
    move |input: Input<'a>| -> ParseResult<'a, usize> {
        let count = input.chars().take_while(|c| *c == ' ' || *c == '\t').count();
        Ok((&input[count..], count))
    }
}

/// Parse inline whitespace
pub fn ws<'a>() -> impl Fn(Input<'a>) -> ParseResult<'a, &'a str> {
    move |input: Input<'a>| -> ParseResult<'a, &'a str> {
        let count = input.chars().take_while(|c| *c == ' ' || *c == '\t').count();
        Ok((&input[count..], &input[..count]))
    }
}

/// Parse a comment (from # to end of line)
pub fn comment<'a>() -> impl Fn(Input<'a>) -> ParseResult<'a, ()> {
    move |input: Input<'a>| -> ParseResult<'a, ()> {
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
pub fn key_value<'a>() -> impl Fn(Input<'a>) -> ParseResult<'a, (String, Value)> {
    move |input: Input<'a>| -> ParseResult<'a, (String, Value)> {
        let key_end = input.find(':').ok_or_else(|| error_at(input, "expected ':' in key-value pair", 0))?;
        let key = input[..key_end].trim().to_string();
        let rest = &input[key_end..];
        
        let after_colon = if rest.starts_with(":") {
            let rest = &rest[1..];
            rest.trim_start()
        } else {
            return Err(error_at(input, "expected ':' after key", 0));
        };
        
        if after_colon.starts_with('\n') || after_colon.is_empty() {
            Ok((after_colon, (key, Value::Map(vec![]))))
        } else {
            let (rest_after_val, value) = scalar()(after_colon)?;
            Ok((rest_after_val, (key, value)))
        }
    }
}

/// Parse a list item: `- value`
pub fn list_item<'a>() -> impl Fn(Input<'a>) -> ParseResult<'a, Value> {
    move |input: Input<'a>| -> ParseResult<'a, Value> {
        if !input.starts_with('-') {
            return Err(error_at(input, "expected list item starting with '-'", 0));
        }
        
        let after_dash = &input[1..];
        let after_space = after_dash.trim_start();
        
        scalar()(after_space)
    }
}

/// Parse a list: multiple `- value` items
pub fn list<'a>() -> impl Fn(Input<'a>) -> ParseResult<'a, Vec<Value>> {
    move |mut input: Input<'a>| -> ParseResult<'a, Vec<Value>> {
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
pub fn block<'a>() -> impl Fn(Input<'a>) -> ParseResult<'a, Vec<(String, Value)>> {
    move |input: Input<'a>| -> ParseResult<'a, Vec<(String, Value)>> {
        let mut pairs = Vec::new();
        let mut remaining = input;
        
        while let Some(colon_pos) = remaining.find(':') {
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
pub fn document<'a>() -> impl Fn(Input<'a>) -> ParseResult<'a, Value> {
    move |input: Input<'a>| -> ParseResult<'a, Value> {
        let remaining = input.trim_start();
        
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
pub fn parse(input: &str) -> Result<Value, parsing::ParseError> {
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

    #[test]
    fn test_comment() {
        let result = comment()("# this is a comment\nrest");
        assert_eq!(result, Ok(("\nrest", ())));
    }

    #[test]
    fn test_document_list() {
        let result = document()("- item1\n- item2");
        assert!(result.is_ok());
        let (_, value) = result.unwrap();
        match value {
            Value::List(items) => assert_eq!(items.len(), 2),
            _ => panic!("Expected list"),
        }
    }

    #[test]
    fn test_document_map() {
        let result = document()("key: value");
        assert!(result.is_ok());
        let (_, value) = result.unwrap();
        match value {
            Value::Map(pairs) => assert_eq!(pairs.len(), 1),
            _ => panic!("Expected map"),
        }
    }

    #[test]
    fn test_parse_complex() {
        let yaml = "name: Alice\nage: 30\nactive: true";
        let result = parse(yaml);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_with_comments() {
        let yaml = "# comment\nname: test";
        let result = parse(yaml);
        assert!(result.is_ok());
    }
