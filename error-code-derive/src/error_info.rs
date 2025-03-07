use darling::{
    FromDeriveInput, FromVariant,
    ast::{Data, Fields, Style},
    util,
};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Generics, Ident, Type};

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(error_info))]
struct ErrorData {
    ident: Ident,
    generics: Generics,
    data: Data<EnumVariants, ()>,
    app_type: Type,
    prefix: String,
}

#[derive(Debug, FromVariant)]
#[darling(attributes(error_info))]
struct EnumVariants {
    ident: syn::Ident,
    fields: Fields<util::Ignored>,
    code: String,
    #[darling(default)]
    app_code: String,
    #[darling(default)]
    client_msg: String,
}

pub(crate) fn process_error_info(input: DeriveInput) -> TokenStream {
    let ErrorData {
        ident: name,
        generics,
        data: Data::Enum(data),
        app_type,
        prefix,
    } = ErrorData::from_derive_input(&input).expect("Can not parse input")
    else {
        panic!("Only enum is supported");
    };

    let code = data
        .iter()
        .map(|variant| {
            let EnumVariants {
                ident,
                fields,
                code,
                app_code,
                client_msg,
            } = variant;
            let code = format!("{}{}", prefix, code);
            let varint_code = match fields.style {
                Style::Struct => quote! { #name::#ident { .. } },
                Style::Tuple => quote! { #name::#ident(_) },
                Style::Unit => quote! { #name::#ident },
            };

            quote! {
                #varint_code => {
                    ErrorInfo::new(
                        #app_code,
                        #code,
                        #client_msg,
                        self,
                    )
                }
            }
        })
        .collect::<Vec<_>>();

    quote! {
        use error_code::{ErrorInfo, ToErrorInfo as _};
        impl #generics ToErrorInfo for #name #generics {
            type T = #app_type;

            fn to_error_info(&self) -> ErrorInfo<Self::T> {
                match self {
                    #(#code),*
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_struct() {
        let input = r#"
        #[derive(thiserror::Error, ToErrorInfo)]
               #[error_info(app_type="http::StatusCode", prefix="01")]
               pub enum MyError {
               #[error("Invalid command: {0}")]
               #[error_info(code="IC", app_code="400")]
               InvalidCommand(String),

               #[error("Invalid argument: {0}")]
               #[error_info(code="IA", app_code="400", client_msg="friendly msg")]
               InvalidArgument(String),

               #[error("{0}")]
               #[error_info(code="RE", app_code="500")]
               RespError(#[from] RespError),
               }
        "#;

        let parsed = syn::parse_str(input).unwrap();
        let info = ErrorData::from_derive_input(&parsed).unwrap();
        println!("info={:#?}", info);

        assert_eq!(info.ident.to_string(), "MyError");
        assert_eq!(info.prefix, "01");

        let code = process_error_info(parsed);
        println!("code={}", code);
    }
}
