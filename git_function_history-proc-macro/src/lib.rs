use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{format_ident, quote_spanned, ToTokens};
use syn::{parse_macro_input, DeriveInput};

/// Allows the type that derives this macro, to have a method from_str
/// that takes a list of strings and returns the type.
///
/// use `[enumstuff(skip)]` attribute on a variant or field to
/// make it not able to be parsed by `from_str`.
#[proc_macro_derive(enumstuff, attributes(enumstuff))]
pub fn enum_stuff(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let span = Span::call_site();
    let name = &ast.ident;
    let vis = &ast.vis;
    let data = match ast.data {
        syn::Data::Enum(data) => data,
        _ => panic!("Only enums are supported"),
    };

    let variants = data.variants.iter().map(|var| {
        let name = var.ident.to_string();
        quote_spanned!(span=>#name=>true,)
    });

    let types = data
        .variants
        .iter()
        .filter_map(|v| {
            // filter out the variants that have the enum attribute #[enum_stuff(skip)]
            for attr in &v.attrs {
                // #[enumstuff(skip)]
                if attr.path().is_ident("enumstuff") {
                    let p = attr.parse_args::<syn::Ident>().unwrap();
                    if p == "skip" {
                        return None;
                    }
                }
            }

            let name_tok = v.ident.to_token_stream();
            let name = name_tok.to_string();
            Some(match v.fields.clone() {
                syn::Fields::Named(_) => {
                    quote_spanned!(span=>#name => None,)
                }
                syn::Fields::Unnamed(v) => {
                    let values = v.unnamed;
                    let (parsers, results): (Vec<_>, Vec<_>) = values
                        .iter()
                        .enumerate()
                        .map(|(i, v)| {
                            let var = format_ident!("f{i}");
                            let rest = quote_spanned!(span=>rest);
                            // todo: more base cases
                            let type_str = v.ty.to_token_stream().to_string();
                            if type_str == "String" {
                                (
                                    quote_spanned!(span=>let (#var, rest) = rest.split_first()?;),
                                    quote_spanned!(span=>#var.to_string()),
                                )
                            } else if type_str == "usize" {
                                (
                                    quote_spanned!(span=>let (#var, rest) = rest.split_first()?;),
                                    quote_spanned!(span=>#var.parse().ok()?),
                                )
                            } else {
                                let ty = v.ty.to_token_stream();
                                (
                                    quote_spanned!(span=>let (#var, rest) = #ty::from_str(#rest)?;),
                                    quote_spanned!(span=>#var),
                                )
                            }
                        })
                        .unzip();
                    quote_spanned!(span=>#name => {
#(#parsers)*
Some((Self::#name_tok(#(#results),*), rest))
                    } )
                }
                syn::Fields::Unit => {
                    let ident = v.ident.to_token_stream();
                    // let rest = quote_spanned!(span=>rest);
                    quote_spanned!(span=>#name => Some((Self::#ident, rest)),)
                }
            })
        })
        .collect::<Vec<_>>();

    let gen = quote_spanned! {
        // TODO: make a function that can recurse through the enum and get the types of the variants and types of the variants variants and so on to a specified depth
        // amd also make a function that can does the same but for the variant names
        span=>
        impl #name {
            // we nned to have a list of the types the variant is constructed of so we can create the variant
            #vis fn from_str<'a, 'b>(makeup: &'b [&'a str]) -> Option<(Self, &'b [&'a str])> {
                let (variant, rest) = makeup.split_first()?;
                match *variant {
                    #(#types)*
                    _ => None,
                }
            }

            #vis fn has_variant(variant: &str) -> bool {
                match variant {
                    #(#variants)*
                    _ => false,
                }
            }
        }
    };
    gen.into()
}
