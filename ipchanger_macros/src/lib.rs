use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse_macro_input,
    punctuated::Punctuated,
    Expr, ExprLit, ItemFn, Lit, MetaNameValue, Token,
};

#[proc_macro_attribute]
pub fn command(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the function first
    let input_fn = parse_macro_input!(item as ItemFn);

    // Parse attribute arguments as a comma-separated list of name-value pairs
    let args = parse_macro_input!(attr with Punctuated<MetaNameValue, Token![,]>::parse_terminated);

    let mut key: Option<char> = None;
    let mut name: Option<String> = None;
    let mut usage: Option<String> = None;
    let mut description: Option<String> = None;

    for nv in args {
        let ident = nv.path.get_ident().map(|i| i.to_string());
        match ident.as_deref() {
            Some("key") => {
                if let Expr::Lit(ExprLit { lit: Lit::Char(c), .. }) = &nv.value {
                    key = Some(c.value());
                }
            }
            Some("name") => {
                if let Expr::Lit(ExprLit { lit: Lit::Str(s), .. }) = &nv.value {
                    name = Some(s.value());
                }
            }
            Some("usage") => {
                if let Expr::Lit(ExprLit { lit: Lit::Str(s), .. }) = &nv.value {
                    usage = Some(s.value());
                }
            }
            Some("description") => {
                if let Expr::Lit(ExprLit { lit: Lit::Str(s), .. }) = &nv.value {
                    description = Some(s.value());
                }
            }
            _ => {}
        }
    }

    let key = match key {
        Some(k) => k,
        None => {
            return syn::Error::new_spanned(&input_fn.sig.ident, "#[command] requires key: 'x'")
                .to_compile_error()
                .into();
        }
    };

    let name = name.unwrap_or_else(|| key.to_string());
    let usage = usage.unwrap_or_else(|| key.to_string());
    let description = description.unwrap_or_default();

    let vis = &input_fn.vis;
    let sig = &input_fn.sig;
    let block = &input_fn.block;
    let fn_name = &input_fn.sig.ident;
    let reg_name = format_ident!("REGISTER_{}", fn_name.to_string().to_uppercase());

    let gen = quote! {
        #vis #sig #block

        #[linkme::distributed_slice(crate::parser::COMMANDS_SLICE)]
        #[linkme(crate = linkme)]
        static #reg_name: crate::parser::Command = crate::parser::Command {
            key: #key,
            name: #name,
            usage: #usage,
            description: #description,
            func: #fn_name,
        };
    };

    gen.into()
}
