extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Ident};

#[proc_macro_attribute]
pub fn derive_data_model(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as DeriveInput);

    input.attrs.push(syn::parse_quote! {
        #[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
    });

    input.attrs.push(syn::parse_quote! {
        #[cfg_attr(
            feature = "integration-tests",
            derive(Debug, fake::Dummy, PartialEq, Eq, Clone)
        )]
    });

    TokenStream::from(quote! { #input })
}

#[proc_macro_attribute]
pub fn taggable(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);

    let name = &input.ident;
    let variants = if let Data::Enum(data_enum) = &input.data {
        &data_enum.variants
    } else {
        panic!("taggable can only be applied to enums");
    };

    let trait_name = Ident::new(&format!("Taggable{}", name), name.span());
    let methods = variants.iter().map(|variant| {
        let variant_name = &variant.ident;

        quote! {
            impl #trait_name for #variant_name {
                fn tag(&self) -> #name {
                    #name::#variant_name
                }
            }
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
