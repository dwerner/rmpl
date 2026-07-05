use mylib::{add, greet};

#[test]
fn test_add_simple() {
    assert_eq!(add(1, 2), 3);
}

#[test]
fn test_add_negative() {
    assert_eq!(add(-1, -1), -2);
}

#[test]
fn test_greet_format() {
    assert_eq!(greet("Test"), "Hello, Test!");
}
