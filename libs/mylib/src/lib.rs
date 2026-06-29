//! MyLib - a simple library for rmpl

pub mod math;

/// Adds two numbers
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

/// Greets the user
pub fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_add() {
        assert_eq!(add(2, 3), 5);
    }
    
    #[test]
    fn test_greet() {
        assert_eq!(greet("World"), "Hello, World!");
    }
}
