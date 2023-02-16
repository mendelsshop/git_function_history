use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(enumstuff, attributes(enumstuff))]
pub fn enum_stuff(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;
    let vis = &ast.vis;
    let data = match ast.data {
        syn::Data::Enum(data) => data,
        _ => panic!("Only enums are supported"),
    };
    let data_type = data
        .variants
        .iter()
        .map(|v| {
            v.fields
                .iter()
                .filter_map(|field| match &field.ty {
                    syn::Type::Path(path) => Some(path.into_token_stream().to_string()),
                    _ => None,
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    let variants_names = data.variants.iter().map(|v| {
        let mut ret = v.ident.to_string();
        for attr in &v.attrs {
            if attr.path.segments.len() == 1 && attr.path.segments[0].ident == "enumstuff" {
                if let Some(proc_macro2::TokenTree::Group(group)) =
                    attr.tokens.clone().into_iter().next()
                {
                    let mut tokens = group.stream().into_iter();
                    if let Some(proc_macro2::TokenTree::Literal(lit)) = tokens.next() {
                        ret = lit.to_string().trim_matches('"').to_string();
                    }
                }
            }
        }
        ret
    });

    let variant_list = data
        .variants
        .iter()
        .map(|v| v.ident.to_string())
        .collect::<Vec<_>>();
    let gen = quote! {
        impl #name {

            #vis fn get_variant_names() -> Vec<&'static str> {
                vec![#(#variants_names),*]
            }

            /// gets the types of the variant ie (u32, u32) for a variant with two fields of type u32
            #vis fn get_variant_types(&self) -> &[&str] {
                match (self.get_variant_name().as_str()) {
                    #(#variant_list =>  &[#(#data_type),*]),*,
                    _ => &[] as &[&str],
                }
            }

            // we nned to have a list of the types the variant is constructed of so we can create the variant
            #vis fn from_str(variant: &str, inside: &[&str]) -> Option<Self> {
                if vec![#(#variant_list),*].contains(&variant) {
                    None
                } else {
                    None
                }
            }

            #vis fn get_variant_types_from_str(variant: &str) -> &[&str] {
                match variant {
                    #(#variant_list =>  &[#(#data_type),*]),*,
                    _ => &[] as &[&str],
                }
            }

            #vis fn get_variant_name(&self) -> String {
                // we cannot just use format!("{:?}", self) because it will return the variant name with its prameters
                // we want to get the variant name without its parameters
                let variant = format!("{:?}", self);
                let mut variant = variant.split("(").collect::<Vec<_>>();
                variant[0].to_string()
            }
        }
    };
    gen.into()
}
