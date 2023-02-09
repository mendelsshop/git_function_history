use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};
/// .
#[proc_macro_derive(enumstuff)]
pub fn enum_stuff(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    // ast.identifier
    let name = &ast.ident;
    let vis = &ast.vis;
    let data = match ast.data {
        syn::Data::Enum(data) => data,
        _ => panic!("Only enums are supported"),
    };
    let variants = data.variants.iter().map(|v| v.ident.to_string());
    let gen = quote! {
        impl #name {
            #vis fn get_variant_names() -> Vec<&'static str> {
                vec![#(#variants),*]
            }

            #vis fn from_str(variant: &str) -> Option<Self> {
                None
            }
        }
    };
    gen.into()
}
