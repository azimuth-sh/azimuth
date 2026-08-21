use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn realizes(_arguments: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn implements_check(_arguments: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn implements_mechanism(_arguments: TokenStream, item: TokenStream) -> TokenStream {
    item
}
