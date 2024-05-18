extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, Data, DeriveInput, Ident, Token,
};

#[proc_macro_attribute]
pub fn derive_data_model(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as DeriveInput);

    input.attrs.push(syn::parse_quote! {
        #[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
    });

    input.attrs.push(syn::parse_quote! {
        #[cfg_attr(
            feature = "integration-tests",
            derive(Debug, PartialEq, Eq)
        )]
    });

    input.attrs.push(syn::parse_quote! {
        #[derive(bitcode::Encode, bitcode::Decode)]
    });

    TokenStream::from(quote! { #input })
}

struct TaggableAttr {
    include: Vec<Ident>,
}

impl Parse for TaggableAttr {
    fn parse(input: ParseStream) -> Result<Self, syn::Error> {
        let mut include = Vec::new();
        while !input.is_empty() {
            include.push(input.parse::<Ident>()?);
            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }
        Ok(TaggableAttr { include })
    }
}

#[proc_macro_attribute]
pub fn taggable(attr: TokenStream, item: TokenStream) -> TokenStream {
    let TaggableAttr { include } = parse_macro_input!(attr as TaggableAttr);
    let input = parse_macro_input!(item as DeriveInput);

    let name = &input.ident;
    let variants = if let Data::Enum(data_enum) = &input.data {
        &data_enum.variants
    } else {
        panic!("taggable can only be applied to enums");
    };

    let trait_name = Ident::new(&format!("Taggable{}", name), name.span());
    let methods = variants.iter().filter_map(|variant| {
        let variant_name = &variant.ident;
        if include.contains(variant_name) {
            Some(quote! {
                impl #trait_name for #variant_name {
                    fn tag(&self) -> #name {
                        #name::#variant_name
                    }
                }
            })
        } else {
            None
        }
    });

    let output = quote! {
        #input

        pub trait #trait_name {
            fn tag(&self) -> #name;
        }

        #(#methods)*
    };

    TokenStream::from(output)
}
