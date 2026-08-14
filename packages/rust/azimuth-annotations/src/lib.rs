use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn realizes(_arguments: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn covers(_arguments: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn implements_mechanism(_arguments: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn covers_mechanism(_arguments: TokenStream, item: TokenStream) -> TokenStream {
    item
}
