//! YAML parser built on the parsing combinator library
//!
//! # Example
//!
//! ```
//! use yaml_parse::{parse, Value};
//!
//! let doc = parse("name: test\nversion: 1").unwrap();
//! ```

pub mod scalar;
pub mod structure;

pub use scalar::{scalar, Value};
pub use structure::{parse, document, key_value, list, block, list_item, indent, ws, comment};

// Re-export types from parsing library
pub use parsing::{Input, ParseResult, ParseError};
