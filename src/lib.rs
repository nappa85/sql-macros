use proc_macro2::Span;
use sqlparser::dialect::MySqlDialect;
use syn::{parse_macro_input, LitStr};

mod select;

#[proc_macro]
pub fn select(
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    // TODO: parse into a structure to manage also dialect and prepared query params
    let query = parse_macro_input!(input as LitStr);
    let into_type = syn::Ident::new("Row", Span::call_site());

    select::parse_select(&MySqlDialect {}, &query.value(), into_type)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
