//! Integration tests for parsing library combinators

use parsing::{tag, take_while, alt, opt, many, many1, pair, tuple3, separated_list, map, value, ignore};

#[test]
fn test_opt_success() {
    let parser = opt(tag("hello"));
    let result = parser("hello world");
    assert_eq!(result, Ok((" world", Some("hello"))));
}

#[test]
fn test_opt_failure_returns_none() {
    let parser = opt(tag("hello"));
    let result = parser("goodbye");
    assert_eq!(result, Ok(("goodbye", None)));
}

#[test]
fn test_many_zero_matches() {
    let parser = many(tag("a"));
    let result = parser("bbb");
    assert_eq!(result, Ok(("bbb", vec![])));
}

#[test]
fn test_many_multiple_matches() {
    let parser = many(tag("a"));
    let result = parser("aaabbb");
    assert_eq!(result, Ok(("bbb", vec!["a", "a", "a"])));
}

#[test]
fn test_many1_requires_at_least_one() {
    let parser = many1(tag("a"));
    let result = parser("bbb");
    assert!(result.is_err());
}

#[test]
fn test_many1_success() {
    let parser = many1(tag("a"));
    let result = parser("aaabbb");
    assert_eq!(result, Ok(("bbb", vec!["a", "a", "a"])));
}

#[test]
fn test_pair_success() {
    let parser = pair(tag("hello"), tag(" world"));
    let result = parser("hello world");
    assert_eq!(result, Ok(("", ("hello", " world"))));
}

#[test]
fn test_pair_failure_second() {
    let parser = pair(tag("hello"), tag(" world"));
    let result = parser("hello, world");
    assert!(result.is_err());
}

#[test]
fn test_tuple3_success() {
    let parser = tuple3(tag("a"), tag("b"), tag("c"));
    let result = parser("abc");
    assert_eq!(result, Ok(("", ("a", "b", "c"))));
}

#[test]
fn test_map_transform() {
    let parser = map(take_while(|c| c.is_alphabetic()), |s: &str| s.to_uppercase());
    let result = parser("Hello123");
    assert_eq!(result, Ok(("123", String::from("HELLO"))));
}

#[test]
fn test_map_parse_number() {
    let parser = map(take_while(|c| c.is_ascii_digit()), |s: &str| s.parse::<u32>().unwrap());
    let result = parser("123abc");
    assert_eq!(result, Ok(("abc", 123)));
}

#[test]
fn test_value_ignore_output() {
    let parser = value(42, tag("hello"));
    let result = parser("hello world");
    assert_eq!(result, Ok((" world", 42)));
}

#[test]
fn test_ignore_discard_result() {
    let parser = ignore(tag("hello"));
    let result = parser("hello world");
    assert_eq!(result, Ok((" world", ())));
}

#[test]
fn test_alt_first_matches() {
    let p1 = tag("hello");
    let p2 = tag("world");
    let parsers = [p1, p2];
    let parser = alt(&parsers);
    let result = parser("hello world");
    assert_eq!(result, Ok((" world", "hello")));
}

#[test]
fn test_alt_second_matches() {
    let p1 = tag("hello");
    let p2 = tag("world");
    let parsers = [p1, p2];
    let parser = alt(&parsers);
    let result = parser("world hello");
    assert_eq!(result, Ok((" hello", "world")));
}

#[test]
fn test_alt_none_match() {
    let p1 = tag("hello");
    let p2 = tag("world");
    let parsers = [p1, p2];
    let parser = alt(&parsers);
    let result = parser("foo bar");
    assert!(result.is_err());
}

#[test]
fn test_separated_list_simple() {
    let parser = separated_list(tag(","), take_while(|c| c != ','));
    let result = parser("a,b,c");
    assert_eq!(result, Ok(("", vec!["a", "b", "c"])));
}

#[test]
fn test_separated_list_single() {
    let parser = separated_list(tag(","), take_while(|c| c != ','));
    let result = parser("a");
    assert_eq!(result, Ok(("", vec!["a"])));
}

#[test]
fn test_separated_list_empty() {
    let parser = separated_list(tag(","), take_while(|c| c != ','));
    let result = parser("123");
    // take_while matches "123" before seeing the comma separator
    assert_eq!(result, Ok(("", vec!["123"])));
}

#[test]
fn test_separated_list_with_spaces() {
    let parser = separated_list(tag(", "), take_while(|c| c != ',' && c != ' '));
    let result = parser("a, b, c");
    assert_eq!(result, Ok(("", vec!["a", "b", "c"])));
}

// Real-world parsing scenarios

#[test]
fn parse_simple_key_value() {
    let key = take_while(|c| c.is_alphabetic());
    let colon = tag(":");
    let value = take_while(|c| c != '\n');
    let kv = pair(pair(key, colon), map(value, |s: &str| s.trim()));
    
    let result = kv("name: value\n");
    assert_eq!(result, Ok(("\n", (("name", ":"), "value"))));
}

#[test]
fn parse_csv_line() {
    let field = take_while(|c| c != ',');
    let parser = separated_list(tag(","), field);
    
    let result = parser("hello,world,foo");
    assert_eq!(result, Ok(("", vec!["hello", "world", "foo"])));
}

#[test]
fn parse_parenthesized_expression() {
    let open = tag("(");
    let close = tag(")");
    let content = take_while(|c| c != ')' && c != '(');
    let parser = pair(pair(open, content), close);
    
    let result = parser("(hello)");
    assert_eq!(result, Ok(("", (("(", "hello"), ")"))));
}

#[test]
fn parse_quoted_string() {
    let content = take_while(|c| c != '"');
    let parser = pair(pair(tag("\""), content), tag("\""));
    
    let result = parser("\"hello world\"");
    assert_eq!(result, Ok(("", (("\"", "hello world"), "\""))));
}
