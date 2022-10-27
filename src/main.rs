use std::collections::HashMap;
use std::convert::Infallible;
use std::error::Error;
use std::fmt::{Display, Formatter, write};
use aws_sdk_dynamodb::Client;
use aws_sdk_dynamodb::error::{GetItemError, GetItemErrorKind};
use aws_sdk_dynamodb::model::{AttributeValue};
use aws_sdk_dynamodb::types::SdkError;
use dynamodb_helper::DynamoDb;
use tokio_stream::StreamExt;

// TODO remove test struct and test db
struct TestStruct {
    partition_key: String,
    value: i32,
    another: Option<String>
}

struct TestDB {
    client: Client,
    table: String,
}

impl TestDB {
    fn new(client: Client, table: String) -> Self {
        TestDB {
            client,
            table,
        }
    }

    // pub async fn scan(&self) -> Result<Vec<TestStruct>, SdkError<ScanError>> {
    //     let items: Result<Vec<std::collections::HashMap<std::string::String, aws_sdk_dynamodb::model::AttributeValue>>, _> = self.client.scan()
    //         .table_name(&self.table)
    //         .into_paginator()
    //         .items()
    //         .send()
    //         .collect()
    //         .await
    //         ;
    //
    //     items
    //         .map(|v| v.iter().map(|i| i.into()).collect())
    //     // Ok(mapped_items)
    // }
}

// #[derive(Debug)]
// enum GetError {
//     ParseError(String),
//     SdkError(aws_sdk_dynamodb::types::SdkError<aws_sdk_dynamodb::error::GetItemError>),
// }
//
// #[derive(Debug)]
// enum GetByPartitionError {
//     ParseError(String),
//     SdkError(aws_sdk_dynamodb::types::SdkError<aws_sdk_dynamodb::error::QueryError>),
// }
//
// #[derive(Debug)]
// enum BatchGetError {
//     ParseError(String),
//     SdkError(aws_sdk_dynamodb::types::SdkError<aws_sdk_dynamodb::error::BatchGetItemError>),
// }
//
// #[derive(Debug)]
// enum ScanError {
//     ParseError(String),
//     SdkError(aws_sdk_dynamodb::types::SdkError<aws_sdk_dynamodb::error::ScanError>),
// }
//
// impl std::fmt::Display for GetError {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         write!(f, "Get error") // TODO
//     }
// }
//
// impl std::error::Error for GetError {}
//
// impl Display for GetByPartitionError {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         write!(f, "Get by partition error") // TODO
//     }
// }
//
// impl std::error::Error for GetByPartitionError {}
//
// impl Display for BatchGetError {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         write!(f, "Batch get error") // TODO
//     }
// }
//
// impl std::error::Error for BatchGetError {}
//
// impl Display for ScanError {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         write!(f, "Scan error") // TODO
//     }
// }
//
// impl std::error::Error for ScanError {}
//
// impl From<aws_sdk_dynamodb::types::SdkError<aws_sdk_dynamodb::error::GetItemError>> for GetError {
//     fn from(err: aws_sdk_dynamodb::types::SdkError<aws_sdk_dynamodb::error::GetItemError>) -> Self {
//         GetError::SdkError(err)
//     }
// }
//
// impl From<aws_sdk_dynamodb::types::SdkError<aws_sdk_dynamodb::error::QueryError>> for GetByPartitionError {
//     fn from(err: aws_sdk_dynamodb::types::SdkError<aws_sdk_dynamodb::error::QueryError>) -> Self {
//         GetByPartitionError::SdkError(err)
//     }
// }
//
// impl From<aws_sdk_dynamodb::types::SdkError<aws_sdk_dynamodb::error::BatchGetItemError>> for BatchGetError {
//     fn from(err: aws_sdk_dynamodb::types::SdkError<aws_sdk_dynamodb::error::BatchGetItemError>) -> Self {
//         BatchGetError::SdkError(err)
//     }
// }
//
// impl From<aws_sdk_dynamodb::types::SdkError<aws_sdk_dynamodb::error::ScanError>> for ScanError {
//     fn from(err: aws_sdk_dynamodb::types::SdkError<aws_sdk_dynamodb::error::ScanError>) -> Self {
//         ScanError::SdkError(err)
//     }
// }

#[derive(Debug)]
enum DynamoDBHelper {
    ParseError(String)
}

impl Display for DynamoDBHelper {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Something")
    }
}

impl Error for DynamoDBHelper {}

impl From<Infallible> for DynamoDBHelper {
    fn from(_: Infallible) -> Self {
        DynamoDBHelper::ParseError("YO".to_string())
    }
}

impl TryFrom<std::collections::HashMap<String, aws_sdk_dynamodb::model::AttributeValue>> for TestStruct {
    type Error = DynamoDBHelper;

    fn try_from(value: HashMap<String, AttributeValue>) -> Result<Self, Self::Error> {
        let first: Option<Result<String, DynamoDBHelper>> = value.get("another")
            .map
            (
                |v| v.as_s().map_err(|_| DynamoDBHelper::ParseError("conversion to s failed".to_string())).map(|v| v.to_string())
            );
        let second = first.transpose()?;

        // first.map(|v| str::parse(v))?)

        // let t = value.get("another").map(|v| v.as_s().map_err(|_| DynamoDBHelper::ParseError("conversion to s failed".to_string()))?
        //     .map(|v| str::parse(v)));


        Ok(TestStruct {
            partition_key: value.get("partition_key").ok_or_else(|| DynamoDBHelper::ParseError("Obligatory not present".to_string()))?.as_s().map_err(|_| DynamoDBHelper::ParseError("conversion to s failed".to_string())).map(|v| str::parse(v))??,
            value: 0,
            another: second,
        })
    }
}

// impl From<TestStruct> for HashMap<String, AttributeValue> {
//     fn from(_: TestStruct) -> Self {
//         todo!()
//     }
// }
//
// impl From<HashMap<String, AttributeValue>> for TestStruct {
//     fn from(_: HashMap<String, AttributeValue>) -> Self {
//         todo!()
//     }
// }
//
// impl From<&HashMap<String, AttributeValue>> for TestStruct {
//     fn from(_: &HashMap<String, AttributeValue>) -> Self {
//         todo!()
//     }
// }

#[tokio::main]
async fn main() {
    #[derive(DynamoDb)]
    pub struct ExampleStruct {
        #[partition]
        partition_key: String,
        a_number: u32,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use super::*;

    #[derive(DynamoDb)]
    #[exclusion("scan", "delete_table", "create_table")]
    pub struct ExampleTestStruct {
        #[partition]
        partition_key: String,
        value: i32,
        true_or_false: bool,
        some_string: String,
        number_liz: Vec<i16>,
        string_list: Vec<String>,
        a_map: HashMap<String, String>,
        optional_string: Option<String>,
        optional_number: Option<i32>,
    }

    #[tokio::test]
    async fn should_generate_a_helper_struct_with_build_method() {
        let _e = ExampleTestStruct {
            partition_key: "example".to_string(),
            value: 0,
            true_or_false: false,
            some_string: "".to_string(),
            number_liz: vec![],
            string_list: vec![],
            a_map: Default::default(),
            optional_string: None,
            optional_number: None
        };
        let _help = ExampleTestStructDb::build(aws_sdk_dynamodb::Region::new("eu-west-1"), "exampleTable").await;
    }
}
