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
    another: Option<String>,
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
}

// #[derive(Debug)]
// enum DynamoDBHelper {
//     ParseError(String)
// }
//
// impl Display for DynamoDBHelper {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         write!(f, "Something")
//     }
// }
//
// impl Error for DynamoDBHelper {}
//
// impl From<Infallible> for DynamoDBHelper {
//     fn from(_: Infallible) -> Self {
//         DynamoDBHelper::ParseError("YO".to_string())
//     }
// }
//
// impl TryFrom<std::collections::HashMap<String, aws_sdk_dynamodb::model::AttributeValue>> for TestStruct {
//     type Error = DynamoDBHelper;
//
//     fn try_from(value: HashMap<String, AttributeValue>) -> Result<Self, Self::Error> {
//         Ok(TestStruct {
//             partition_key: value.get("partition_key").ok_or_else(|| DynamoDBHelper::ParseError("Obligatory not present".to_string()))?.as_s().map_err(|_| DynamoDBHelper::ParseError("conversion to s failed".to_string())).map(|v| str::parse(v))??,
//             value: 0,
//             another: value.get("another").map(|v| v.as_s().map_err(|_| DynamoDBHelper::ParseError("conversion to s failed".to_string())).map(|v| v.to_string())).transpose()?,
//         })
//     }
// }

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
            optional_number: None,
        };
        let _help = ExampleTestStructDb::build(aws_sdk_dynamodb::Region::new("eu-west-1"), "exampleTable").await;
    }
}
