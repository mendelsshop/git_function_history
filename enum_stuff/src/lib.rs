use proc_macro::TokenStream;
use proc_macro2::Literal;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};
/// .
#[proc_macro_derive(enumstuff, attributes(enumstuff))]
pub fn enum_stuff(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    // ast.identifier
    let name = &ast.ident;
    let vis = &ast.vis;
    let data = match ast.data {
        syn::Data::Enum(data) => data,
        _ => panic!("Only enums are supported"),
    };
    let variants_names = data.variants.iter().map(|v| {
        let mut ret = v.ident.to_string();
        for attr in &v.attrs {
            if attr.path.segments.len() == 1 && attr.path.segments[0].ident == "enumstuff" {
                if let Some(proc_macro2::TokenTree::Group(group)) = attr.tokens.clone().into_iter().next() {
                    let mut tokens = group.stream().into_iter();
                    if let Some(proc_macro2::TokenTree::Literal(lit)) = tokens.next() {
                       ret =  lit.to_string().trim_matches('"').to_string();
                    }
                }
            }
        }
        ret});
    let variant_list = data.variants.iter().map(|v| {v.ident.to_string()}).collect::<Vec<_>>();
    let gen = quote! {
        impl #name {
            #vis fn get_variant_names() -> Vec<&'static str> {
                vec![#(#variants_names),*]
            }

            #vis fn from_str(variant: &str) -> Option<Self> {
                if vec![#(#variant_list),*].contains(&variant) {
                    None
                } else {
                    None
                }
            }
        }
    };
    gen.into()
}
