use quote::__private::Ident;
use quote::quote;
use std::fmt::{Display, Formatter};
use std::error::Error;

// TODO if I have to do it like this, make the name specific for the struct!
// TODO the display...
pub fn generate_helper_error(error_name: &Ident) -> proc_macro2::TokenStream {
    quote! {
        #[derive(Debug)]
        pub enum #error_name {
            ParseError(String),
            GetError(aws_sdk_dynamodb::types::SdkError<aws_sdk_dynamodb::error::GetItemError>),
            GetByPartitionError(aws_sdk_dynamodb::types::SdkError<aws_sdk_dynamodb::error::QueryError>),
            BatchGetError(aws_sdk_dynamodb::types::SdkError<aws_sdk_dynamodb::error::BatchGetItemError>),
            PutError(aws_sdk_dynamodb::types::SdkError<aws_sdk_dynamodb::error::PutItemError>),
            DeleteError(aws_sdk_dynamodb::types::SdkError<aws_sdk_dynamodb::error::DeleteItemError>),
            ScanError(aws_sdk_dynamodb::types::SdkError<aws_sdk_dynamodb::error::ScanError>),
            CreateTableError(aws_sdk_dynamodb::types::SdkError<aws_sdk_dynamodb::error::CreateTableError>),
            DeleteTableError(aws_sdk_dynamodb::types::SdkError<aws_sdk_dynamodb::error::DeleteTableError>),
        }

        impl std::fmt::Display for #error_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "DynamoDB Helper Error")
            }
        }

        impl std::error::Error for #error_name {}

        impl From<aws_sdk_dynamodb::types::SdkError<aws_sdk_dynamodb::error::GetItemError>> for #error_name {
            fn from(err: aws_sdk_dynamodb::types::SdkError<aws_sdk_dynamodb::error::GetItemError>) -> Self {
                #error_name::GetError(err)
            }
        }

        impl From<aws_sdk_dynamodb::types::SdkError<aws_sdk_dynamodb::error::QueryError>> for #error_name {
            fn from(err: aws_sdk_dynamodb::types::SdkError<aws_sdk_dynamodb::error::QueryError>) -> Self {
                #error_name::GetByPartitionError(err)
            }
        }

        impl From<aws_sdk_dynamodb::types::SdkError<aws_sdk_dynamodb::error::BatchGetItemError>> for #error_name {
            fn from(err: aws_sdk_dynamodb::types::SdkError<aws_sdk_dynamodb::error::BatchGetItemError>) -> Self {
                #error_name::BatchGetError(err)
            }
        }

        impl From<aws_sdk_dynamodb::types::SdkError<aws_sdk_dynamodb::error::PutItemError>> for #error_name {
            fn from(err: aws_sdk_dynamodb::types::SdkError<aws_sdk_dynamodb::error::PutItemError>) -> Self {
                #error_name::PutError(err)
            }
        }

        impl From<aws_sdk_dynamodb::types::SdkError<aws_sdk_dynamodb::error::DeleteItemError>> for #error_name {
            fn from(err: aws_sdk_dynamodb::types::SdkError<aws_sdk_dynamodb::error::DeleteItemError>) -> Self {
                #error_name::DeleteError(err)
            }
        }

        impl From<aws_sdk_dynamodb::types::SdkError<aws_sdk_dynamodb::error::CreateTableError>> for #error_name {
            fn from(err: aws_sdk_dynamodb::types::SdkError<aws_sdk_dynamodb::error::CreateTableError>) -> Self {
                #error_name::CreateTableError(err)
            }
        }

        impl From<aws_sdk_dynamodb::types::SdkError<aws_sdk_dynamodb::error::DeleteTableError>> for #error_name {
            fn from(err: aws_sdk_dynamodb::types::SdkError<aws_sdk_dynamodb::error::DeleteTableError>) -> Self {
                #error_name::DeleteTableError(err)
            }
        }

        impl From<aws_sdk_dynamodb::types::SdkError<aws_sdk_dynamodb::error::ScanError>> for #error_name {
            fn from(err: aws_sdk_dynamodb::types::SdkError<aws_sdk_dynamodb::error::ScanError>) -> Self {
                #error_name::ScanError(err)
            }
        }
    }
}
