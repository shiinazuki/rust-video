use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

pub(crate) fn process_enum_from(input: DeriveInput) -> TokenStream {
    let ident = input.ident;

    let generics = input.generics;

    let variants = match input.data {
        syn::Data::Enum(data) => data.variants,
        _ => panic!("only support on enum"),
    };

    let from_impls = variants.iter().map(|variant| {
        let var_ident = &variant.ident;

        match &variant.fields {
            syn::Fields::Unnamed(fields) => {
                if fields.unnamed.len() != 1 {
                    quote! {}
                } else {
                    let field = fields.unnamed.first().expect("should have one field");
                    let ty = &field.ty;

                    quote! {
                        impl #generics From<#ty> for #ident #generics {
                            fn from(value: #ty) -> Self {
                                #ident::#var_ident(value)
                            }
                        }
                    }
                }
            }
            syn::Fields::Named(_fields) => quote! {},
            syn::Fields::Unit => quote! {},
        }
    });

    quote! {
        #(#from_impls)*
    }
}

