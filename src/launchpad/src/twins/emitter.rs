extern crate proc_macro;
use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(EventEmitter)]
fn derive(input: TokenStream) -> TokenStream {
    let ast = parse_token_stream!(input);
    println!("here {:#?}", ast);
    TokenStream::new()
}
