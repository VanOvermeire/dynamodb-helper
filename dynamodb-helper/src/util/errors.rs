use quote::__private::Ident;
use quote::quote;
use std::fmt::{Debug, Display, Formatter};
use std::error::Error;
use aws_sdk_dynamodb::error::GetItemError;
use aws_sdk_dynamodb::types::SdkError;
use proc_macro2::TokenStream;

pub fn generate_error_names(helper_name: &Ident) -> (proc_macro2::Ident, proc_macro2::Ident, proc_macro2::Ident, proc_macro2::Ident, proc_macro2::Ident) {
    let get_error = Ident::new(&format!("{}GetError", helper_name), helper_name.span());
    let get_by_partition_error = Ident::new(&format!("{}GetByPartitionError", helper_name), helper_name.span());
    let batch_get_error = Ident::new(&format!("{}BatchGetError", helper_name), helper_name.span());
    let scan_error = Ident::new(&format!("{}ScanError", helper_name), helper_name.span());
    let parse_error = Ident::new(&format!("{}ParseError", helper_name), helper_name.span());

    (get_error, get_by_partition_error, batch_get_error, scan_error, parse_error)
}

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

    let parse_error_stream = generate_parse_error(&parse_error);

    quote! {
        #parse_error_stream
        #(#impl_errors)*
    }
}

fn generate_parse_error(parse_error: &Ident) -> proc_macro2::TokenStream {
    quote! {
        #[derive(Debug)]
        pub struct #parse_error {
            message: String,
        }

        impl std::fmt::Display for #parse_error {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "Parse error: {}", self.message)
            }
        }

        impl std::error::Error for #parse_error {}

        impl From<std::convert::Infallible> for #parse_error {
            fn from(_: std::convert::Infallible) -> Self {
                #parse_error {
                    message: "Could parse attribute into string".to_string()
                }
            }
        }
    }
}

fn generate_impl_error(error: &Ident, aws_error: &Ident, parse_error: &Ident) -> proc_macro2::TokenStream {
    let error_name = error.to_string();

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

        impl std::fmt::Display for #error {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    #error::ParseError(val) => write!(f, "{} parse error: {}", &#error_name, val),
                    #error::AwsError(val) => write!(f, "{} aws error {}", &#error_name, val)
                }
            }
        }
    }
}
