//! Generic parser combinator library
//!
//! A minimal, function-based parser combinator library for building
//! parsers without external dependencies.
//!
//! # Example
//!
//! ```
//! use parsing::{tag, take_while, many, pair};
//!
//! // Parse a simple CSV line
//! let field = take_while(|c| c != ',');
//! let csv_line = many(pair(field, tag(",")));
//!
//! let result = csv_line("hello,world,test");
//! assert!(result.is_ok());
//! ```

pub mod error;
pub mod input;
pub mod primitives;
pub mod combinators;

// Re-export main types
pub use error::ParseError;
pub use input::{Input, ParseResult, get_position, error_at};
pub use primitives::{
    tag, one_of, take_while, take_while1, take_until, take, eof,
    newline, comment,
};
pub use combinators::{
    alt, opt, many, many1, pair, tuple3, separated_list,
    value, ignore, context, map,
};
