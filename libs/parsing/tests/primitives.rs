//! Integration tests for parsing library primitives

use parsing::{tag, one_of, take_while, take_while1, take_until, take, eof, newline, comment};

#[test]
fn test_tag_exact_match() {
    let result = tag("hello")("hello");
    assert_eq!(result, Ok(("", "hello")));
}

#[test]
fn test_tag_partial_match() {
    let result = tag("hello")("hello world");
    assert_eq!(result, Ok((" world", "hello")));
}

#[test]
fn test_tag_no_match() {
    let result = tag("hello")("goodbye");
    assert!(result.is_err());
}

#[test]
fn test_tag_empty() {
    let result = tag("")("anything");
    assert_eq!(result, Ok(("anything", "")));
}

#[test]
fn test_one_of_first_char() {
    let result = one_of("aeiou")("apple");
    assert_eq!(result, Ok(("pple", 'a')));
}

#[test]
fn test_one_of_middle_char() {
    let result = one_of("xyz")("xyz");
    assert_eq!(result, Ok(("yz", 'x')));
}

#[test]
fn test_one_of_no_match() {
    let result = one_of("aeiou")("xyz");
    assert!(result.is_err());
}

#[test]
fn test_take_while_alphabetic() {
    let result = take_while(|c| c.is_alphabetic())("hello123");
    assert_eq!(result, Ok(("123", "hello")));
}

#[test]
fn test_take_while_digits() {
    let result = take_while(|c| c.is_ascii_digit())("123abc");
    assert_eq!(result, Ok(("abc", "123")));
}

#[test]
fn test_take_while_none() {
    let result = take_while(|c| c.is_alphabetic())("123abc");
    assert_eq!(result, Ok(("123abc", "")));
}

#[test]
fn test_take_while_all() {
    let result = take_while(|c| c.is_alphabetic())("hello");
    assert_eq!(result, Ok(("", "hello")));
}

#[test]
fn test_take_while1_requires_match() {
    let result = take_while1(|c| c.is_alphabetic())("123abc");
    assert!(result.is_err());
}

#[test]
fn test_take_while1_success() {
    let result = take_while1(|c| c.is_alphabetic())("hello123");
    assert_eq!(result, Ok(("123", "hello")));
}

#[test]
fn test_take_until_found() {
    let result = take_until(":")("key:value");
    assert_eq!(result, Ok((":value", "key")));
}

#[test]
fn test_take_until_not_found() {
    let result = take_until(":")("no colon");
    assert!(result.is_err());
}

#[test]
fn test_take_exact() {
    let result = take(5)("hello world");
    assert_eq!(result, Ok((" world", "hello")));
}

#[test]
fn test_take_all() {
    let result = take(5)("hello");
    assert_eq!(result, Ok(("", "hello")));
}

#[test]
fn test_eof_empty() {
    let result = eof()("");
    assert_eq!(result, Ok(("", ())));
}

#[test]
fn test_eof_not_empty() {
    let result = eof()("not empty");
    assert!(result.is_err());
}

#[test]
fn test_newline_success() {
    let result = newline()("\n");
    assert_eq!(result, Ok(("", "\n")));
}

#[test]
fn test_newline_with_rest() {
    let result = newline()("\nhello");
    assert_eq!(result, Ok(("hello", "\n")));
}

#[test]
fn test_newline_failure() {
    let result = newline()("hello");
    assert!(result.is_err());
}

#[test]
fn test_comment_simple() {
    let result = comment()("# comment\nrest");
    assert_eq!(result, Ok(("\nrest", "# comment")));
}

#[test]
fn test_comment_no_newline() {
    let result = comment()("# comment");
    assert_eq!(result, Ok(("", "# comment")));
}

#[test]
fn test_comment_failure() {
    let result = comment()("not a comment");
    assert!(result.is_err());
}
