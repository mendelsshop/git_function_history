use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{quote_spanned, ToTokens};
use syn::{parse_macro_input, DeriveInput, Type};

#[proc_macro_derive(enumstuff, attributes(enumstuff))]
pub fn enum_stuff(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;
    let vis = &ast.vis;
    let data = match ast.data {
        syn::Data::Enum(data) => data,
        _ => panic!("Only enums are supported"),
    };
    let data_type_filtered = data
        .variants
        .iter()
        .map(|v| {
            v.fields
                .iter()
                .filter_map(|field| {
                    // see if the variant has the enum attribute #[enum_stuff(skip)]
                    // use v to get the attributes of the variant

                    for attr in &v.attrs {
                        // #[enumstuff(skip)]
                        if attr.path().is_ident("enumstuff") {
                            let p = attr.parse_args::<syn::Ident>().unwrap();
                            if p == "skip" {
                                return None;
                            }
                        }
                    }

                    match &field.ty {
                        syn::Type::Path(path) => Some(path),
                        _ => None,
                    }
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    let data_type = data
        .variants
        .iter()
        .map(|v| {
            v.fields
                .iter()
                .filter_map(|field| {
                    // see if the variant has the enum attribute #[enum_stuff(skip)]
                    // use v to get the attributes of the variant
                    match &field.ty {
                        syn::Type::Path(path) => Some(path),
                        _ => None,
                    }
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    // element in data_type but not in data_type_filtered
    let data_type_rest = data_type
        .iter()
        .zip(data_type_filtered.iter())
        .map(|(a, b)| {
            a.iter()
                .filter(|x| !b.contains(x))
                .map(|x| x.to_token_stream().to_string())
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    let types = data
        .variants
        .iter()
        .filter_map(|v| {
            let tts = match v.fields.clone() {
                syn::Fields::Named(n) => n.into_token_stream(),
                syn::Fields::Unnamed(u) => u.into_token_stream(),
                syn::Fields::Unit => return None,
            };
            syn::parse2::<Type>(tts).ok()
        })
        .collect::<Vec<_>>();

    let data_type_sr = data_type
        .iter()
        .map(|v| {
            v.iter()
                .map(|t| {
                    let mut ret = t.to_token_stream().to_string();
                    if ret.starts_with("std::option::Option<") {
                        ret = ret.replace("std::option::Option<", "");
                        ret.pop();
                    }
                    ret
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let variants_names = data
        .variants
        .iter()
        .map(|v| {
            let mut ret = v.ident.to_string();
            // #[enumstuff(name = "PLFilter")]
            for attr in &v.attrs {
                if attr.path().is_ident("enumstuff") {}
            }
            ret
        })
        .collect::<Vec<_>>();

    let variant_list = data
        .variants
        .iter()
        .map(|v| v.ident.to_string())
        .collect::<Vec<_>>();
    let span = Span::call_site();
    let gen = quote_spanned! {
        // TODO: make a function that can recurse through the enum and get the types of the variants and types of the variants variants and so on to a specified depth
        // amd also make a function that can does the same but for the variant names
        span=>
        impl #name {

            #vis fn get_variant_names() -> &'static [&'static str] {
                &[#(#variants_names),*]
            }

            #vis fn get_variant_names_recurse(list: &[&str]) -> Option<Vec<&'static str>> {
                let mut list = list.iter();
                let variants = list.next()?.clone();
                // if the first element of the list is not a variant then we return None
                // if !Self::get_variant_names().contains(variants) {
                //     return None;
                // }
                // // if the then we access the variant and get its variants
                // let variants = Self::get_variant_types_from_str(variants);
                // // if the list is empty then we return the variants of the variant
                // if list.next().is_none() {
                //     return Some(variants);
                // }

                // let ret = vec![];
                // // then we need to go do the same thing for each variant in variants
                // // but we need to turn variant into an actual type so we can call get_variant_names_recurse on the inner variant
                // for variant in variants {
                //     // use syn::parse2::<Type>(tts).ok()
                //     // to turn variant into a type
                //     // then call get_variant_names_recurse on the type
                //     // and return the result
                //     let variant = syn::parse2::<Type>(variant.into()).ok()?;
                //     let variant = variant.get_variant_names_recurse(list);
                //     if let Some(variant) = variant {
                //         ret.extend(variant);
                //     }

                // }
                use std::collections::HashSet;
                match variants {
                    #(#variant_list => {
                        // turn to hashset so we can compare to the filtered data_type
                        // [#(#data_type),*].iter().collect::<HashSet<_>>().difference(&[#(#data_type_filtered),*]);
                        let  mut slice: Vec<&'static str> = vec![];
                        // recurse to next level list
                        #(#data_type_filtered::get_variant_names_recurse(list.cloned().collect::<Vec<_>>().as_slice()).map(|x| slice.extend(x));)*
                        #(slice.push(#data_type_rest);)*
                        if slice.len() <= 0 {
                            return None;
                        }
                        Some(slice)
                    }),*,
                    _ => None
                }



            }

            /// gets the types of the variant ie (u32, u32) for a variant with two fields of type u32
            #vis fn get_variant_types(&self) -> &'static [&'static str] {
                match (self.get_variant_name().as_str()) {
                    #(#variant_list =>  &[#(#data_type_sr),*]),*,
                    _ => &[] as &[&str],
                }
            }

            // /// makes an into function for each variant
            // #(#vis fn #variant_list(variant: &str) -> Option<#types> {
            //     if let #variant_list = variant {
            //         Some(#types)
            //     } else {
            //         None
            //     }
            // })*

            // we nned to have a list of the types the variant is constructed of so we can create the variant
            #vis fn from_str(variant: &str, inside: &[&str]) -> Option<Self> {
                if vec![#(#variant_list),*].contains(&variant) {
                    None
                } else {
                    None
                }
            }

            #vis fn get_variant_types_from_str(variant: &str) -> &'static [&'static str] {
                match variant {
                    #(#variant_list =>  &[#(#data_type_sr),*]),*,
                    var => {
                        &[] as &[&'static str]},
                }
            }

            // #vis fn get_variant_types_from_str_recurse(variant: &str, depth: usize) -> &'static [&'static str] {
            //     match variant {
            //         #(#variant_list =>  &[#(#data_type),*]),*,
            //         var => {
            //             &[] as &[&str]},
            //     }
            // }

            #vis fn get_variant_name(&self) -> String {
                // we cannot just use format!("{:?}", self) because it will return the variant name with its prameters
                // we want to get the variant name without its parameters
                format!("{:?}", self).split("(").next().unwrap_or("").to_string()
            }
        }
    };
    gen.into()
}
