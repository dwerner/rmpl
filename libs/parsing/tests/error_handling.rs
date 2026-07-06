//! Error handling tests for parsing library

use parsing::{ParseError, get_position, error_at};

#[test]
fn test_error_creation() {
    let err = ParseError::new("test error", 5, 10);
    assert_eq!(err.message, "test error");
    assert_eq!(err.line, 5);
    assert_eq!(err.column, 10);
}

#[test]
fn test_error_display() {
    let err = ParseError::new("unexpected token", 3, 7);
    assert_eq!(err.to_string(), "unexpected token at line 3, column 7");
}

#[test]
fn test_error_format() {
    let err = ParseError::new("parse failed", 1, 1);
    assert_eq!(err.format(), "parse failed at line 1, column 1");
}

#[test]
fn test_get_position_start() {
    let input = "hello";
    let (line, col) = get_position(input, 0);
    assert_eq!(line, 1);
    assert_eq!(col, 1);
}

#[test]
fn test_get_position_middle_of_line() {
    let input = "hello world";
    let (line, col) = get_position(input, 6);
    assert_eq!(line, 1);
    assert_eq!(col, 7);
}

#[test]
fn test_get_position_end_of_line() {
    let input = "hello";
    let (line, col) = get_position(input, 5);
    assert_eq!(line, 1);
    assert_eq!(col, 6);
}

#[test]
fn test_get_position_after_newline() {
    let input = "hello\nworld";
    let (line, col) = get_position(input, 6);
    assert_eq!(line, 2);
    assert_eq!(col, 1);
}

#[test]
fn test_get_position_multiline() {
    let input = "line1\nline2\nline3";
    let (line, col) = get_position(input, 13);
    assert_eq!(line, 3);
    assert_eq!(col, 2); // 'i' in line3
}

#[test]
fn test_get_position_last_char() {
    let input = "abc";
    let (line, col) = get_position(input, 2);
    assert_eq!(line, 1);
    assert_eq!(col, 3);
}

#[test]
fn test_get_position_empty_input() {
    let input = "";
    let (line, col) = get_position(input, 0);
    assert_eq!(line, 1);
    assert_eq!(col, 1);
}

#[test]
fn test_error_at_basic() {
    let input = "hello";
    let err = error_at(input, "test", 2);
    assert_eq!(err.message, "test");
    assert_eq!(err.line, 1);
    assert_eq!(err.column, 3);
}

#[test]
fn test_error_at_multiline() {
    let input = "hello\nworld";
    let err = error_at(input, "test", 7);
    assert_eq!(err.line, 2);
    assert_eq!(err.column, 2);
}

#[test]
fn test_error_clone() {
    let err1 = ParseError::new("test", 1, 1);
    let err2 = err1.clone();
    assert_eq!(err1.message, err2.message);
    assert_eq!(err1.line, err2.line);
    assert_eq!(err1.column, err2.column);
}

#[test]
fn test_error_partial_eq() {
    let err1 = ParseError::new("test", 1, 1);
    let err2 = ParseError::new("test", 1, 1);
    let err3 = ParseError::new("test", 2, 1);
    
    assert_eq!(err1, err2);
    assert_ne!(err1, err3);
}
