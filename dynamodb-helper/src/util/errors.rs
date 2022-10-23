use std::fmt::{Display, Formatter};
use aws_sdk_dynamodb::types::SdkError;
use aws_sdk_dynamodb::error::*;
use std::error::Error;

#[derive(Debug)]
pub enum DynamoDBHelperError {
    ParseError(String),
    GetError(SdkError<GetItemError>),
    GetByPartitionError(SdkError<QueryError>),
    BatchGetError(SdkError<BatchGetItemError>),
    PutError(SdkError<PutItemError>),
    DeleteError(SdkError<DeleteItemError>),
    ScanError(SdkError<ScanError>),
    CreateTableError(SdkError<CreateTableError>),
    DeleteTableError(SdkError<DeleteTableError>),
}

impl Display for DynamoDBHelperError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "DynamoDB Helper Error") // TODO
    }
}

impl Error for DynamoDBHelperError {}

impl From<SdkError<GetItemError>> for DynamoDBHelperError {
    fn from(err: SdkError<GetItemError>) -> Self {
        DynamoDBHelperError::GetError(err)
    }
}

impl From<SdkError<QueryError>> for DynamoDBHelperError {
    fn from(err: SdkError<QueryError>) -> Self {
        DynamoDBHelperError::GetByPartitionError(err)
    }
}

impl From<SdkError<BatchGetItemError>> for DynamoDBHelperError {
    fn from(err: SdkError<BatchGetItemError>) -> Self {
        DynamoDBHelperError::BatchGetError(err)
    }
}

impl From<SdkError<PutItemError>> for DynamoDBHelperError {
    fn from(err: SdkError<PutItemError>) -> Self {
        DynamoDBHelperError::PutError(err)
    }
}

impl From<SdkError<DeleteItemError>> for DynamoDBHelperError {
    fn from(err: SdkError<DeleteItemError>) -> Self {
        DynamoDBHelperError::DeleteError(err)
    }
}

impl From<SdkError<CreateTableError>> for DynamoDBHelperError {
    fn from(err: SdkError<CreateTableError>) -> Self {
        DynamoDBHelperError::CreateTableError(err)
    }
}

impl From<SdkError<DeleteTableError>> for DynamoDBHelperError {
    fn from(err: SdkError<DeleteTableError>) -> Self {
        DynamoDBHelperError::DeleteTableError(err)
    }
}

impl From<SdkError<ScanError>> for DynamoDBHelperError {
    fn from(err: SdkError<ScanError>) -> Self {
        DynamoDBHelperError::ScanError(err)
    }
}
