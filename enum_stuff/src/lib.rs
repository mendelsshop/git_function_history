use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{quote_spanned, ToTokens};
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(enumstuff, attributes(enumstuff))]
pub fn enum_stuff(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let span = Span::mixed_site();
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
                    let v = values
                        .iter()
                        .map(|v| {
                            let rest = quote_spanned!(span=>rest);
                            // todo: more base cases
                            if v.ty.to_token_stream().to_string() == "String" {
                                quote_spanned!(span=>#rest.join(" "))
                            } else {
                                let ty = v.ty.to_token_stream();
                                quote_spanned!(span=>#ty::from_str(#rest)?)
                            }
                        })
                        .collect::<Vec<_>>();
                    quote_spanned!(span=>#name => Some(Self::#name_tok(#(#v),*)),)
                }
                syn::Fields::Unit => {
                    let ident = v.ident.to_token_stream();
                    quote_spanned!(span=>#name => Some(Self::#ident),)
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
            #vis fn from_str(makeup: &[&str]) -> Option<Self> {
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
