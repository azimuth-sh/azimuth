use proc_macro::{TokenStream, TokenTree};

fn entity_marker(arguments: TokenStream, item: TokenStream) -> TokenStream {
    let mut tokens = arguments.into_iter();
    let Some(TokenTree::Literal(literal)) = tokens.next() else {
        return "compile_error!(\"Azimuth marker requires one stable entity ID string literal\");"
            .parse()
            .unwrap();
    };
    if tokens.next().is_some() {
        return "compile_error!(\"Azimuth marker requires exactly one argument\");"
            .parse()
            .unwrap();
    }
    let literal = literal.to_string();
    if !(literal.starts_with('"') || literal.starts_with('r')) {
        return "compile_error!(\"Azimuth marker requires a string literal\");"
            .parse()
            .unwrap();
    }
    let value = literal.find('"').and_then(|start| {
        literal
            .rfind('"')
            .filter(|end| *end > start)
            .map(|end| &literal[start + 1..end])
    });
    if !value.is_some_and(|value| {
        !value.is_empty()
            && !value.starts_with('-')
            && !value.ends_with('-')
            && value
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
    }) {
        return "compile_error!(\"Azimuth entity ID must be one lowercase kebab segment\");"
            .parse()
            .unwrap();
    }
    item
}

#[proc_macro_attribute]
pub fn realizes(arguments: TokenStream, item: TokenStream) -> TokenStream {
    entity_marker(arguments, item)
}

#[proc_macro_attribute]
pub fn implements_check(arguments: TokenStream, item: TokenStream) -> TokenStream {
    entity_marker(arguments, item)
}

#[proc_macro_attribute]
pub fn implements_mechanism(arguments: TokenStream, item: TokenStream) -> TokenStream {
    entity_marker(arguments, item)
}
