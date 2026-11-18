use proc_macro::{Delimiter, TokenStream, TokenTree};
use std::str::FromStr;

#[proc_macro]
pub fn generate_style_getters(input: TokenStream) -> TokenStream {
    let mut tokens = input.into_iter();
    let mut output = String::new();

    while let Some(token) = tokens.next() {
        let category = match token {
            TokenTree::Ident(ident) => ident.to_string(),
            _ => {
                return compile_error("Expected category identifier");
            }
        };

        let fields = match tokens.next() {
            Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Brace => {
                group.stream()
            }
            _ => return compile_error("Expected fields"),
        };

        output.push_str(&generate_category_getters(&category, fields));
    }

    TokenStream::from_str(&output).unwrap_or_else(|_| compile_error("Failed to generate getters"))
}

fn generate_category_getters(category: &str, fields: TokenStream) -> String {
    let mut output = format!("impl {category} {{");

    let mut current = Vec::new();

    for token in fields {
        if matches!(&token, TokenTree::Punct(p) if p.as_char() == ',') {
            generate_getter(&mut output, &current);
            current.clear();
        } else {
            current.push(token);
        }
    }

    if !current.is_empty() {
        generate_getter(&mut output, &current);
    }

    output.push('}');
    output
}

fn generate_getter(output: &mut String, tokens: &[TokenTree]) {
    let Some(colon) = tokens
        .iter()
        .position(|t| matches!(t, TokenTree::Punct(p) if p.as_char() == ':'))
    else {
        return;
    };

    let Some(arrow) = tokens.windows(2).position(|pair| {
        matches!(&pair[0], TokenTree::Punct(p) if p.as_char() == '=')
            && matches!(&pair[1], TokenTree::Punct(p) if p.as_char() == '>')
    }) else {
        return;
    };

    let Some(name) = tokens.first().and_then(|t| {
        if let TokenTree::Ident(id) = t {
            Some(id.to_string())
        } else {
            None
        }
    }) else {
        return;
    };

    let field_type: TokenStream = tokens[colon + 1..arrow].iter().cloned().collect();

    let default: TokenStream = tokens[arrow + 2..].iter().cloned().collect();

    output.push_str(&format!(
        "pub fn get_{name}(&self) -> {field_type} {{
            self.{name}.clone().unwrap_or({default})
        }}"
    ));
}

fn compile_error(message: &str) -> TokenStream {
    TokenStream::from_str(&format!("compile_error!({message:?});")).unwrap()
}
