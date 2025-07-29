use crate::{tokenstream_or_empty_if_no_retrieval_methods, BATCH_GET_METHOD_NAME, GET_METHOD_NAME, SCAN_METHOD_NAME};
use proc_macro2::Ident;
use quote::quote;

pub fn generate_error_names(helper_name: &Ident) -> (Ident, Ident, Ident, Ident, Ident) {
    let get_error = Ident::new(&format!("{helper_name}GetError"), helper_name.span());
    let get_by_partition_error = Ident::new(&format!("{helper_name}GetByPartitionError"), helper_name.span());
    let batch_get_error = Ident::new(&format!("{helper_name}BatchGetError"), helper_name.span());
    let scan_error = Ident::new(&format!("{helper_name}ScanError"), helper_name.span());
    let parse_error = Ident::new(&format!("{helper_name}ParseError"), helper_name.span());

    (get_error, get_by_partition_error, batch_get_error, scan_error, parse_error)
}

pub fn generate_helper_error(struct_name: &Ident, exclusions: &[&str]) -> proc_macro2::TokenStream {
    let (get_error, get_by_partition_error, batch_get_error, scan_error, parse_error) = generate_error_names(struct_name);

    let error_copies = [
        (
            &get_error,
            Ident::new("get_item", struct_name.span()),
            Ident::new("GetItemError", struct_name.span()),
            GET_METHOD_NAME,
        ),
        (
            &get_by_partition_error,
            Ident::new("query", struct_name.span()),
            Ident::new("QueryError", struct_name.span()),
            GET_METHOD_NAME,
        ),
        (
            &batch_get_error,
            Ident::new("batch_get_item", struct_name.span()),
            Ident::new("BatchGetItemError", struct_name.span()),
            BATCH_GET_METHOD_NAME,
        ),
        (
            &scan_error,
            Ident::new("scan", struct_name.span()),
            Ident::new("ScanError", struct_name.span()),
            SCAN_METHOD_NAME,
        ),
    ];

    let impl_errors = error_copies
        .iter()
        .filter(|error_name| !exclusions.contains(&error_name.3))
        .map(|error_name| generate_impl_error(error_name.0, &error_name.1, &error_name.2, &parse_error));

    let parse_error_stream = tokenstream_or_empty_if_no_retrieval_methods(generate_parse_error(&parse_error), exclusions);

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

        impl #parse_error {
            pub fn new(message: String) -> #parse_error {
                #parse_error {
                    message,
                }
            }
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

fn generate_impl_error(error: &Ident, error_package: &Ident, aws_error: &Ident, parse_error: &Ident) -> proc_macro2::TokenStream {
    let error_name = error.to_string();

    quote! {
        #[derive(Debug)]
        pub enum #error {
            ParseError(String),
            AwsError(aws_sdk_dynamodb::error::SdkError<aws_sdk_dynamodb::operation::#error_package::#aws_error>),
        }

        impl std::error::Error for #error {}

        impl From<aws_sdk_dynamodb::error::SdkError<aws_sdk_dynamodb::operation::#error_package::#aws_error>> for #error {
            fn from(err: aws_sdk_dynamodb::error::SdkError<aws_sdk_dynamodb::operation::#error_package::#aws_error>) -> Self {
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
