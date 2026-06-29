//! A simple hello macro - a procedural macro crate
//! This version doesn't require syn/quote - just basic token manipulation

extern crate proc_macro;

use proc_macro::TokenStream;

/// A simple identity macro that just returns the input
/// This is a function-like proc macro
#[proc_macro]
pub fn hello(item: TokenStream) -> TokenStream {
    // For now, just echo back the input
    // In a real implementation, you'd transform the tokens
    let input = item.to_string();
    
    // Create a simple greeting
    let output = format!(
        "{{ fn hello_macro() {{ println!(\"Hello from {}!\"); }} }}",
        input
    );
    
    output.parse().unwrap()
}
