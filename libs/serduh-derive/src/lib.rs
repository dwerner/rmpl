//! Derive macros for serduh serialization framework
//! 
//! Uses custom token parsing and quote-lite for code generation

extern crate proc_macro;
extern crate serduh_core;

use proc_macro::TokenStream;
use quote_lite::{quote, tokens_to_string};

/// Token type for parsing
#[derive(Debug, Clone)]
enum Token {
    Ident(String),
    Punct(char),
    Group(Vec<Token>),
    Literal(String),
}

/// Parse proc_macro TokenStream into our token representation
fn parse_tokens(input: TokenStream) -> Vec<Token> {
    let mut tokens = Vec::new();
    for tt in input {
        tokens.push(token_tree_to_token(tt));
    }
    tokens
}

fn token_tree_to_token(tt: proc_macro::TokenTree) -> Token {
    match tt {
        proc_macro::TokenTree::Ident(ident) => Token::Ident(ident.to_string()),
        proc_macro::TokenTree::Punct(punct) => Token::Punct(punct.as_char()),
        proc_macro::TokenTree::Group(group) => {
            let mut inner = Vec::new();
            for tt in group.stream() {
                inner.push(token_tree_to_token(tt));
            }
            Token::Group(inner)
        }
        proc_macro::TokenTree::Literal(lit) => Token::Literal(lit.to_string()),
    }
}

/// Parse a struct definition to extract name and field names
fn parse_struct_def(tokens: &[Token]) -> Result<(String, Vec<String>), String> {
    let mut iter = tokens.iter().peekable();
    
    // Skip to 'struct' keyword
    while let Some(token) = iter.next() {
        if let Token::Ident(name) = token {
            if name == "struct" {
                // Next should be the struct name
                if let Some(Token::Ident(struct_name)) = iter.next() {
                    // Look for brace group
                    while let Some(token) = iter.next() {
                        if let Token::Group(ref inner) = token {
                            let fields = parse_fields(inner)?;
                            return Ok((struct_name.clone(), fields));
                        }
                    }
                    return Ok((struct_name.clone(), Vec::new()));
                }
            }
        }
    }
    
    Err("Expected struct definition".to_string())
}

/// Parse fields from a brace group
fn parse_fields(tokens: &[Token]) -> Result<Vec<String>, String> {
    let mut fields = Vec::new();
    let mut i = 0;
    
    while i < tokens.len() {
        if let Token::Ident(name) = &tokens[i] {
            // Check if followed by ':'
            if i + 1 < tokens.len() {
                if let Token::Punct(':') = tokens[i + 1] {
                    fields.push(name.clone());
                    // Skip to next comma or end
                    i += 2;
                    while i < tokens.len() {
                        if let Token::Punct(',') | Token::Punct('}') = tokens[i] {
                            break;
                        }
                        i += 1;
                    }
                    continue;
                }
            }
        }
        i += 1;
    }
    
    Ok(fields)
}

/// Derive Serialize for a struct
#[proc_macro_derive(Serialize, attributes(serduh))]
pub fn derive_serialize(input: TokenStream) -> TokenStream {
    let tokens = parse_tokens(input);
    
    let result = match parse_struct_def(&tokens) {
        Ok((name, fields)) => {
            if fields.is_empty() {
                // Unit struct
                quote!(
                    impl serduh_core::Serialize for #name {
                        fn serialize<S: serduh_core::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                            serializer.serialize_unit_struct(#name)
                        }
                    }
                )
            } else {
                // Struct with named fields - generate field serialization
                let field_names: Vec<&str> = fields.iter().map(|s| s.as_str()).collect();
                quote!(
                    impl serduh_core::Serialize for #name {
                        fn serialize<S: serduh_core::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                            let mut state = serializer.serialize_struct(#name, 0)?;
                            #(state.serialize_field(#field_names, &self.#field_names)?;)*
                            state.end()
                        }
                    }
                )
            }
        }
        Err(e) => {
            quote!(compile_error!(#e))
        }
    };
    
    // Convert QuoteStream to proc_macro::TokenStream
    let s = tokens_to_string(&result);
    s.parse().unwrap()
}

/// Derive Deserialize for a struct  
#[proc_macro_derive(Deserialize, attributes(serduh))]
pub fn derive_deserialize(input: TokenStream) -> TokenStream {
    let tokens = parse_tokens(input);
    
    let result = match parse_struct_def(&tokens) {
        Ok((name, _fields)) => {
            quote!(
                impl serduh_core::DeserializeOwned for #name {
                    fn deserialize<D: serduh_core::Deserializer>(deserializer: D) -> Result<Self, D::Error> {
                        unimplemented!()
                    }
                }
            )
        }
        Err(e) => {
            quote!(compile_error!(#e))
        }
    };
    
    let s = tokens_to_string(&result);
    s.parse().unwrap()
}
