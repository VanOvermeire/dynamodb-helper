use quote::__private::Ident;
use quote::quote;
use std::fmt::{Debug, Display, Formatter};
use std::error::Error;
use aws_sdk_dynamodb::error::GetItemError;
use aws_sdk_dynamodb::types::SdkError;

pub fn generate_error_names(helper_name: &Ident) -> (proc_macro2::Ident, proc_macro2::Ident, proc_macro2::Ident, proc_macro2::Ident) {
    let get_error = Ident::new(&format!("{}GetError", helper_name), helper_name.span());
    let get_by_partition_error = Ident::new(&format!("{}GetByPartitionError", helper_name), helper_name.span());
    let batch_get_error = Ident::new(&format!("{}BatchGetError", helper_name), helper_name.span());
    let scan_error = Ident::new(&format!("{}ScanError", helper_name), helper_name.span());

    (get_error, get_by_partition_error, batch_get_error, scan_error)
}

// TODO the display...
pub fn generate_helper_error(struct_name: &Ident) -> proc_macro2::TokenStream {
    let (get_error, get_by_partition_error, batch_get_error, scan_error) = generate_error_names(struct_name);

    quote! {
        #[derive(Debug)]
        enum #get_error {
            ParseError(String),
            AwsError(aws_sdk_dynamodb::types::SdkError<aws_sdk_dynamodb::error::GetItemError>),
        }

        #[derive(Debug)]
        enum #get_by_partition_error {
            ParseError(String),
            AwsError(aws_sdk_dynamodb::types::SdkError<aws_sdk_dynamodb::error::QueryError>),
        }

        #[derive(Debug)]
        enum #batch_get_error {
            ParseError(String),
            AwsError(aws_sdk_dynamodb::types::SdkError<aws_sdk_dynamodb::error::BatchGetItemError>),
        }

        #[derive(Debug)]
        enum #scan_error {
            ParseError(String),
            AwsError(aws_sdk_dynamodb::types::SdkError<aws_sdk_dynamodb::error::ScanError>),
        }

        impl std::fmt::Display for #get_error {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(f, "Get error") // TODO
            }
        }

        impl std::error::Error for #get_error {}

        impl Display for #get_by_partition_error {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(f, "Get by partition error") // TODO
            }
        }

        impl std::error::Error for #get_by_partition_error {}

        impl Display for #batch_get_error {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(f, "Batch get error") // TODO
            }
        }

        impl std::error::Error for #batch_get_error {}

        impl Display for #scan_error {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(f, "Scan error") // TODO
            }
        }

        impl std::error::Error for #scan_error {}

        impl From<aws_sdk_dynamodb::types::SdkError<aws_sdk_dynamodb::error::GetItemError>> for #get_error {
            fn from(err: aws_sdk_dynamodb::types::SdkError<aws_sdk_dynamodb::error::GetItemError>) -> Self {
                #get_error::AwsError(err)
            }
        }

        impl From<aws_sdk_dynamodb::types::SdkError<aws_sdk_dynamodb::error::QueryError>> for #get_by_partition_error {
            fn from(err: aws_sdk_dynamodb::types::SdkError<aws_sdk_dynamodb::error::QueryError>) -> Self {
                #get_by_partition_error::AwsError(err)
            }
        }

        impl From<aws_sdk_dynamodb::types::SdkError<aws_sdk_dynamodb::error::BatchGetItemError>> for #batch_get_error {
            fn from(err: aws_sdk_dynamodb::types::SdkError<aws_sdk_dynamodb::error::BatchGetItemError>) -> Self {
                #batch_get_error::AwsError(err)
            }
        }

        impl From<aws_sdk_dynamodb::types::SdkError<aws_sdk_dynamodb::error::ScanError>> for #scan_error {
            fn from(err: aws_sdk_dynamodb::types::SdkError<aws_sdk_dynamodb::error::ScanError>) -> Self {
                #scan_error::AwsError(err)
            }
        }
    }

    // quote! {
    //     #[derive(Debug)]
    //     pub enum #error_name {
    //         ParseError(String),
    //         GetError(aws_sdk_dynamodb::types::SdkError<aws_sdk_dynamodb::error::GetItemError>),
    //         GetByPartitionError(aws_sdk_dynamodb::types::SdkError<aws_sdk_dynamodb::error::QueryError>),
    //         BatchGetError(aws_sdk_dynamodb::types::SdkError<aws_sdk_dynamodb::error::BatchGetItemError>),
    //         PutError(aws_sdk_dynamodb::types::SdkError<aws_sdk_dynamodb::error::PutItemError>),
    //         DeleteError(aws_sdk_dynamodb::types::SdkError<aws_sdk_dynamodb::error::DeleteItemError>),
    //         ScanError(aws_sdk_dynamodb::types::SdkError<aws_sdk_dynamodb::error::ScanError>),
    //         CreateTableError(aws_sdk_dynamodb::types::SdkError<aws_sdk_dynamodb::error::CreateTableError>),
    //         DeleteTableError(aws_sdk_dynamodb::types::SdkError<aws_sdk_dynamodb::error::DeleteTableError>),
    //     }
    //
    //     impl std::fmt::Display for #error_name {
    //         fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    //             write!(f, "DynamoDB Helper Error")
    //         }
    //     }
    //
    //     impl std::error::Error for #error_name {}
    //
    //     impl From<aws_sdk_dynamodb::types::SdkError<aws_sdk_dynamodb::error::GetItemError>> for #error_name {
    //         fn from(err: aws_sdk_dynamodb::types::SdkError<aws_sdk_dynamodb::error::GetItemError>) -> Self {
    //             #error_name::GetError(err)
    //         }
    //     }
    //
    //     impl From<aws_sdk_dynamodb::types::SdkError<aws_sdk_dynamodb::error::QueryError>> for #error_name {
    //         fn from(err: aws_sdk_dynamodb::types::SdkError<aws_sdk_dynamodb::error::QueryError>) -> Self {
    //             #error_name::GetByPartitionError(err)
    //         }
    //     }
    //
    //     impl From<aws_sdk_dynamodb::types::SdkError<aws_sdk_dynamodb::error::BatchGetItemError>> for #error_name {
    //         fn from(err: aws_sdk_dynamodb::types::SdkError<aws_sdk_dynamodb::error::BatchGetItemError>) -> Self {
    //             #error_name::BatchGetError(err)
    //         }
    //     }
    //
    //     impl From<aws_sdk_dynamodb::types::SdkError<aws_sdk_dynamodb::error::PutItemError>> for #error_name {
    //         fn from(err: aws_sdk_dynamodb::types::SdkError<aws_sdk_dynamodb::error::PutItemError>) -> Self {
    //             #error_name::PutError(err)
    //         }
    //     }
    //
    //     impl From<aws_sdk_dynamodb::types::SdkError<aws_sdk_dynamodb::error::DeleteItemError>> for #error_name {
    //         fn from(err: aws_sdk_dynamodb::types::SdkError<aws_sdk_dynamodb::error::DeleteItemError>) -> Self {
    //             #error_name::DeleteError(err)
    //         }
    //     }
    //
    //     impl From<aws_sdk_dynamodb::types::SdkError<aws_sdk_dynamodb::error::CreateTableError>> for #error_name {
    //         fn from(err: aws_sdk_dynamodb::types::SdkError<aws_sdk_dynamodb::error::CreateTableError>) -> Self {
    //             #error_name::CreateTableError(err)
    //         }
    //     }
    //
    //     impl From<aws_sdk_dynamodb::types::SdkError<aws_sdk_dynamodb::error::DeleteTableError>> for #error_name {
    //         fn from(err: aws_sdk_dynamodb::types::SdkError<aws_sdk_dynamodb::error::DeleteTableError>) -> Self {
    //             #error_name::DeleteTableError(err)
    //         }
    //     }
    //
    //     impl From<aws_sdk_dynamodb::types::SdkError<aws_sdk_dynamodb::error::ScanError>> for #error_name {
    //         fn from(err: aws_sdk_dynamodb::types::SdkError<aws_sdk_dynamodb::error::ScanError>) -> Self {
    //             #error_name::ScanError(err)
    //         }
    //     }
    // }
}
