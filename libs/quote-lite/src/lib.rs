//! Minimal Rust code generation for proc macros
//! 
//! Provides template-based code generation as a lightweight alternative to `quote`.

use std::fmt;

/// A token in the output stream
#[derive(Debug, Clone)]
pub enum Token {
    Ident(String),
    Punct(char),
    Literal(String),
    Group(Delimiter, Vec<Token>),
}

/// Delimiter types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Delimiter {
    None,
    Parenthesis,
    Brace,
    Bracket,
}

/// Token stream builder
#[derive(Debug, Clone, Default)]
pub struct TokenStream {
    tokens: Vec<Token>,
}

impl TokenStream {
    pub fn new() -> Self {
        Self { tokens: Vec::new() }
    }
    
    pub fn push_ident(&mut self, s: &str) {
        self.tokens.push(Token::Ident(s.to_string()));
    }
    
    pub fn push_punct(&mut self, c: char) {
        self.tokens.push(Token::Punct(c));
    }
    
    pub fn push_literal(&mut self, s: &str) {
        self.tokens.push(Token::Literal(s.to_string()));
    }
    
    pub fn push_group(&mut self, delim: Delimiter, inner: TokenStream) {
        self.tokens.push(Token::Group(delim, inner.tokens));
    }
    
    pub fn push(&mut self, token: Token) {
        self.tokens.push(token);
    }
    
    pub fn extend(&mut self, other: TokenStream) {
        self.tokens.extend(other.tokens);
    }
    
    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }
}

/// Trait for types that can be quoted into a TokenStream
pub trait ToTokens {
    fn to_tokens(&self, tokens: &mut TokenStream);
}

impl ToTokens for TokenStream {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(self.clone());
    }
}

impl ToTokens for str {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.push_ident(self);
    }
}

impl ToTokens for String {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.push_ident(self);
    }
}

impl ToTokens for Ident {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.push_ident(&self.0);
    }
}

/// Simple identifier wrapper
#[derive(Debug, Clone)]
pub struct Ident(pub String);

impl Ident {
    pub fn new(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// Builder for generating code
pub struct Tokens {
    stream: TokenStream,
}

impl Tokens {
    pub fn new() -> Self {
        Self {
            stream: TokenStream::new(),
        }
    }
    
    pub fn ident(mut self, s: &str) -> Self {
        self.stream.push_ident(s);
        self
    }
    
    pub fn punct(mut self, c: char) -> Self {
        self.stream.push_punct(c);
        self
    }
    
    pub fn literal(mut self, s: &str) -> Self {
        self.stream.push_literal(s);
        self
    }
    
    pub fn group(mut self, delim: Delimiter, inner: TokenStream) -> Self {
        self.stream.push_group(delim, inner);
        self
    }
    
    pub fn tokens(mut self, ts: TokenStream) -> Self {
        self.stream.extend(ts);
        self
    }
    
    pub fn build(self) -> TokenStream {
        self.stream
    }
}

impl Default for Tokens {
    fn default() -> Self {
        Self::new()
    }
}

/// Format a TokenStream as a string (for debugging)
pub fn tokens_to_string(tokens: &TokenStream) -> String {
    tokens
        .tokens
        .iter()
        .map(|t| match t {
            Token::Ident(s) => s.clone(),
            Token::Punct(c) => c.to_string(),
            Token::Literal(s) => format!("\"{}\"", s),
            Token::Group(delim, inner) => {
                let delim_str = match delim {
                    Delimiter::Parenthesis => "(",
                    Delimiter::Brace => "{",
                    Delimiter::Bracket => "[",
                    Delimiter::None => "",
                };
                let end_str = match delim {
                    Delimiter::Parenthesis => ")",
                    Delimiter::Brace => "}",
                    Delimiter::Bracket => "]",
                    Delimiter::None => "",
                };
                let inner_str = inner
                    .iter()
                    .map(|t| match t {
                        Token::Ident(s) => s.clone(),
                        Token::Punct(c) => c.to_string(),
                        Token::Literal(s) => format!("\"{}\"", s),
                        Token::Group(d, inner) => {
                            format!(
                                "{}{}{}",
                                match d {
                                    Delimiter::Parenthesis => "(",
                                    Delimiter::Brace => "{",
                                    Delimiter::Bracket => "[",
                                    Delimiter::None => "",
                                },
                                inner
                                    .iter()
                                    .map(|t| match t {
                                        Token::Ident(s) => s.clone(),
                                        Token::Punct(c) => c.to_string(),
                                        Token::Literal(s) => format!("\"{}\"", s),
                                        Token::Group(_, _) => String::new(),
                                    })
                                    .collect::<Vec<_>>()
                                    .join(" "),
                                match d {
                                    Delimiter::Parenthesis => ")",
                                    Delimiter::Brace => "}",
                                    Delimiter::Bracket => "]",
                                    Delimiter::None => "",
                                }
                            )
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(" ");
                format!("{}{}{}", delim_str, inner_str, end_str)
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

impl TokenStream {
    pub fn iter(&self) -> std::slice::Iter<Token> {
        self.tokens.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_tokens() {
        let ts = Tokens::new()
            .ident("fn")
            .ident("main")
            .group(Delimiter::Parenthesis, TokenStream::new())
            .group(Delimiter::Brace, TokenStream::new())
            .build();
        let s = tokens_to_string(&ts);
        assert!(s.contains("fn"));
        assert!(s.contains("main"));
    }
    
    #[test]
    fn test_impl_block() {
        let mut ts = TokenStream::new();
        ts.push_ident("impl");
        ts.push_ident("MyType");
        ts.push_punct('{');
        ts.push_ident("fn");
        ts.push_ident("method");
        ts.push_punct('{');
        ts.push_punct('}');
        ts.push_punct('}');
        let s = tokens_to_string(&ts);
        assert!(s.contains("impl"));
        assert!(s.contains("MyType"));
    }

    #[test]
    fn test_tokenstream_new() {
        let ts = TokenStream::new();
        assert!(ts.is_empty());
    }

    #[test]
    fn test_tokenstream_push_ident() {
        let mut ts = TokenStream::new();
        ts.push_ident("foo");
        assert!(!ts.is_empty());
        assert_eq!(ts.tokens.len(), 1);
    }

    #[test]
    fn test_tokenstream_push_punct() {
        let mut ts = TokenStream::new();
        ts.push_punct('=');
        assert_eq!(ts.tokens.len(), 1);
        match &ts.tokens[0] {
            Token::Punct(c) => assert_eq!(*c, '='),
            _ => panic!("Expected Punct"),
        }
    }

    #[test]
    fn test_tokenstream_push_literal() {
        let mut ts = TokenStream::new();
        ts.push_literal("hello");
        match &ts.tokens[0] {
            Token::Literal(s) => assert_eq!(s, "hello"),
            _ => panic!("Expected Literal"),
        }
    }

    #[test]
    fn test_tokenstream_extend() {
        let mut ts1 = TokenStream::new();
        ts1.push_ident("fn");
        ts1.push_ident("main");

        let mut ts2 = TokenStream::new();
        ts2.push_punct('(');
        ts2.push_punct(')');

        ts1.extend(ts2);
        assert_eq!(ts1.tokens.len(), 4);
    }

    #[test]
    fn test_tokens_builder() {
        let ts = Tokens::new()
            .ident("struct")
            .ident("Point")
            .punct('{')
            .ident("x")
            .punct(':')
            .ident("i32")
            .punct('}')
            .build();

        let s = tokens_to_string(&ts);
        assert!(s.contains("struct"));
        assert!(s.contains("Point"));
        assert!(s.contains("x"));
        assert!(s.contains("i32"));
    }

    #[test]
    fn test_tokens_with_group() {
        let inner = TokenStream::new();
        let ts = Tokens::new()
            .ident("vec")
            .group(Delimiter::Bracket, inner)
            .build();

        let s = tokens_to_string(&ts);
        assert!(s.contains("vec"));
        assert!(s.contains("["));
    }

    #[test]
    fn test_to_tokens_trait_str() {
        let mut ts = TokenStream::new();
        "foo".to_tokens(&mut ts);
        match &ts.tokens[0] {
            Token::Ident(s) => assert_eq!(s, "foo"),
            _ => panic!("Expected Ident"),
        }
    }

    #[test]
    fn test_to_tokens_trait_string() {
        let mut ts = TokenStream::new();
        "bar".to_string().to_tokens(&mut ts);
        match &ts.tokens[0] {
            Token::Ident(s) => assert_eq!(s, "bar"),
            _ => panic!("Expected Ident"),
        }
    }

    #[test]
    fn test_to_tokens_trait_tokenstream() {
        let mut ts1 = TokenStream::new();
        ts1.push_ident("hello");

        let mut ts2 = TokenStream::new();
        ts2.push_ident("world");

        ts1.to_tokens(&mut ts2);
        assert_eq!(ts2.tokens.len(), 2);
    }

    #[test]
    fn test_ident_new() {
        let ident = Ident::new("my_ident");
        assert_eq!(ident.0, "my_ident");
    }

    #[test]
    fn test_tokens_default() {
        let tokens: Tokens = Default::default();
        assert!(tokens.build().is_empty());
    }

    #[test]
    fn test_tokens_chaining() {
        let ts = Tokens::new()
            .ident("impl")
            .punct('<')
            .ident("T")
            .punct(',')
            .ident("U")
            .punct('>')
            .ident("for")
            .ident("MyType")
            .build();

        let s = tokens_to_string(&ts);
        assert!(s.contains("impl"));
        assert!(s.contains("<"));
        assert!(s.contains("T"));
        assert!(s.contains("U"));
        assert!(s.contains(">"));
        assert!(s.contains("for"));
        assert!(s.contains("MyType"));
    }

    #[test]
    fn test_nested_groups() {
        let inner = TokenStream::new();
        let middle = Tokens::new()
            .ident("inner")
            .group(Delimiter::Parenthesis, inner)
            .build();

        let outer = Tokens::new()
            .ident("outer")
            .group(Delimiter::Brace, middle)
            .build();

        let s = tokens_to_string(&outer);
        assert!(s.contains("outer"));
        assert!(s.contains("{"));
        assert!(s.contains("inner"));
    }

    #[test]
    fn test_tokens_to_string_empty() {
        let ts = TokenStream::new();
        let s = tokens_to_string(&ts);
        assert_eq!(s, "");
    }

    #[test]
    fn test_tokens_to_string_with_punct() {
        let mut ts = TokenStream::new();
        ts.push_ident("a");
        ts.push_punct('+');
        ts.push_ident("b");
        let s = tokens_to_string(&ts);
        assert!(s.contains("a"));
        assert!(s.contains("+"));
        assert!(s.contains("b"));
    }

    #[test]
    fn test_all_delimiters() {
        let inner = TokenStream::new();
        let ts = Tokens::new()
            .group(Delimiter::Parenthesis, inner.clone())
            .group(Delimiter::Brace, inner.clone())
            .group(Delimiter::Bracket, inner.clone())
            .group(Delimiter::None, inner)
            .build();

        let s = tokens_to_string(&ts);
        assert!(s.contains("("));
        assert!(s.contains(")"));
        assert!(s.contains("{"));
        assert!(s.contains("}"));
        assert!(s.contains("["));
        assert!(s.contains("]"));
    }

    #[test]
    fn test_iter() {
        let mut ts = TokenStream::new();
        ts.push_ident("a");
        ts.push_ident("b");
        ts.push_ident("c");

        let count = ts.iter().count();
        assert_eq!(count, 3);
    }
}
