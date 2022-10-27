use quote::__private::Ident;
use quote::quote;
use std::fmt::{Debug, Display, Formatter};
use std::error::Error;
use aws_sdk_dynamodb::error::GetItemError;
use aws_sdk_dynamodb::types::SdkError;

pub fn generate_error_names(helper_name: &Ident) -> (proc_macro2::Ident, proc_macro2::Ident, proc_macro2::Ident, proc_macro2::Ident, proc_macro2::Ident) {
    let get_error = Ident::new(&format!("{}GetError", helper_name), helper_name.span());
    let get_by_partition_error = Ident::new(&format!("{}GetByPartitionError", helper_name), helper_name.span());
    let batch_get_error = Ident::new(&format!("{}BatchGetError", helper_name), helper_name.span());
    let scan_error = Ident::new(&format!("{}ScanError", helper_name), helper_name.span());
    let parse_error = Ident::new(&format!("{}ParseError", helper_name), helper_name.span());

    (get_error, get_by_partition_error, batch_get_error, scan_error, parse_error)
}

// TODO the display... (could maybe also generate that one like the other stuff)
pub fn generate_helper_error(struct_name: &Ident) -> proc_macro2::TokenStream {
    let (get_error, get_by_partition_error, batch_get_error, scan_error, parse_error) = generate_error_names(struct_name);

    let error_copies = [
        (&get_error, Ident::new("GetItemError", struct_name.span())),
        (&get_by_partition_error, Ident::new("QueryError", struct_name.span())),
        (&batch_get_error, Ident::new("BatchGetItemError", struct_name.span())),
        (&scan_error, Ident::new("ScanError", struct_name.span())),
    ];
    let impl_errors = error_copies
        .iter()
        .map(|error_names| generate_impl_error(error_names.0, &error_names.1, &parse_error));

    quote! {
        #(#impl_errors)*

        impl std::fmt::Display for #get_error {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(f, "Get error") // TODO
            }
        }

        impl Display for #get_by_partition_error {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(f, "Get by partition error") // TODO
            }
        }

        impl Display for #batch_get_error {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(f, "Batch get error") // TODO
            }
        }

        impl Display for #scan_error {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(f, "Scan error") // TODO
            }
        }

        #[derive(Debug)]
        pub struct #parse_error {
            message: String,
        }

        impl Display for #parse_error {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(f, "Parse error") // TODO
            }
        }

        impl std::error::Error for #parse_error {}

        impl From<Infallible> for #parse_error {
            fn from(_: Infallible) -> Self {
                #parse_error {
                    message: "Could parse attribute into string".to_string()
                }
            }
        }
    }
}

fn generate_impl_error(error: &Ident, aws_error: &Ident, parse_error: &Ident) -> proc_macro2::TokenStream {
    quote! {
        #[derive(Debug)]
        pub enum #error {
            ParseError(String),
            AwsError(aws_sdk_dynamodb::types::SdkError<aws_sdk_dynamodb::error::#aws_error>),
        }

        impl std::error::Error for #error {}

        impl From<aws_sdk_dynamodb::types::SdkError<aws_sdk_dynamodb::error::#aws_error>> for #error {
            fn from(err: aws_sdk_dynamodb::types::SdkError<aws_sdk_dynamodb::error::#aws_error>) -> Self {
                #error::AwsError(err)
            }
        }

        impl From<#parse_error> for #error {
            fn from(err: #parse_error) -> Self {
                #error::ParseError(err.message)
            }
        }
    }
}
