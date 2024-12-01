use std::collections::HashMap;

use proc_macro2::{Span, TokenStream};
use quote::quote;
use sqlparser::{ast::{Expr, Ident, SelectItem, SetExpr, Statement, TableFactor}, dialect::Dialect, parser::Parser};

type TableDict<'a> = HashMap<&'a str, &'a str>;

struct FieldProvenance<'a> {
    table: &'a str,
    field: &'a str,
    alias: Option<&'a str>,
}

pub fn parse_select(dialect: &impl Dialect, query: &str, into_type: syn::Ident) -> syn::Result<TokenStream> {
    let query = Parser::parse_sql(dialect, query).map_err(|e| syn::Error::new(Span::call_site(), e))?;
    if let &[Statement::Query(ref query)] = query.as_slice() {
        parse_select_body(&query.body, into_type)
    } else {
        Err(syn::Error::new(Span::call_site(), "Only a single select query is supported"))
    }
}

fn parse_select_body(body: &SetExpr, into_type: syn::Ident) -> syn::Result<TokenStream> {
    match body {
        SetExpr::Select(select) => {
            // build alias table
            let tables = select.from.iter().filter_map(|twj| if let TableFactor::Table { ref name, ref alias, .. } = &twj.relation {
                Some((extract_name(&name.0, alias.as_ref().map(|alias| &alias.name)), extract_name(&name.0, None)))
            } else {
                None
            }).collect::<TableDict>();

            // map selected fields
            let fields= select.projection.iter().map(|item| match item {
                SelectItem::UnnamedExpr(expr) => parse_expr(&tables, expr, None),
                SelectItem::ExprWithAlias { expr, alias } => parse_expr(&tables, expr, Some(alias)),
                _ => todo!(),
            }).collect::<syn::Result<Vec<_>>>()?;

            let fields = fields.into_iter().map(|field| {
                let table_trait = syn::Ident::new(&format!("{}Schema", heck::AsPascalCase(field.table)), Span::call_site());
                let field_name = syn::Ident::new(&heck::AsSnakeCase(field.alias.unwrap_or(field.field)).to_string(), Span::call_site());
                let associated_type_name = syn::Ident::new(&heck::AsShoutySnakeCase(field.field).to_string(), Span::call_site());
                quote! {
                    #field_name: #table_trait::#associated_type_name::parse()?,
                }
            }).collect::<TokenStream>();
            
            // dilemma here: how do we manage field typing in a static way? Trait's associated types? Struct's methods?
            Ok(quote!{
                Ok(#into_type {
                    #fields
                })
            })
        },
        SetExpr::Query(_query) => todo!("Handle select subquery"),
        _ => Err(syn::Error::new(Span::call_site(), "Not a select query")),
    }
}

fn extract_name<'a>(tablename: &'a [Ident], alias: Option<&'a Ident>) -> &'a str {
    alias.map(|ident| &ident.value).unwrap_or(tablename.last().map(|ident| &ident.value).expect("There should be at least one part"))
}

fn parse_expr<'a>(tables: &TableDict<'a>, expr: &'a Expr, alias: Option<&'a Ident>) -> syn::Result<FieldProvenance<'a>> {
    match expr {
        Expr::Identifier(ident) => {
            // TODO: here we should scan all included tables to find if there is only one field with that name, for now just pretend we're querying a single table
            let table = tables.values().next().copied().ok_or_else(|| syn::Error::new(Span::call_site(), "Apparently there are no tables involved in this query"))?;
            Ok(FieldProvenance { table, field: &ident.value, alias: alias.map(|alias| alias.value.as_str()) })
        },
        Expr::CompoundIdentifier(idents) => {
            let field = extract_name(idents, None);
            let table_alias = idents.first().map(|ident| ident.value.as_str()).ok_or_else(|| syn::Error::new(Span::call_site(), "Malformed compound identifier"))?;
            let table = tables.get(table_alias).copied().ok_or_else(|| syn::Error::new(Span::call_site(), format!("No table \"{table_alias}\" found in this query")))?;
            Ok(FieldProvenance { table, field, alias: alias.map(|alias| alias.value.as_str()) })
        },
        _ => todo!()
    }
}
