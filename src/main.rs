use std::collections::HashMap;
use aws_sdk_dynamodb::Client;
use aws_sdk_dynamodb::model::{AttributeValue};
use dynamodb_helper::DynamoDb;
use tokio_stream::StreamExt;

// TODO remove test struct and test db
struct TestStruct {
    partition_key: String,
    value: i32,
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

impl From<TestStruct> for HashMap<String, AttributeValue> {
    fn from(_: TestStruct) -> Self {
        todo!()
    }
}

impl From<HashMap<String, AttributeValue>> for TestStruct {
    fn from(_: HashMap<String, AttributeValue>) -> Self {
        todo!()
    }
}

impl From<&HashMap<String, AttributeValue>> for TestStruct {
    fn from(_: &HashMap<String, AttributeValue>) -> Self {
        todo!()
    }
}

#[tokio::main]
async fn main() {
    #[derive(DynamoDb)]
    pub struct ExampleStruct {
        #[partition]
        partition_key: String,
        val: bool,
    }
    // let e = ExampleStruct {
    //     partition_key: "".to_string(),
    //     val: false,
    // };
    // let t: bool = AttributeValue::Bool(false).as_bool().map(|b| *b).expect("lol");
    // let map: HashMap<String, AttributeValue> = e.into();
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use super::*;

    #[derive(DynamoDb)]
    struct ExampleTestStruct {
        #[partition]
        partition_key: String,
        value: i32,
    }

    #[tokio::test]
    async fn should_generate_a_helper_struct_with_build_method() {
        let e = ExampleTestStruct {
            partition_key: "example".to_string(),
            value: 0
        };
        let help = ExampleTestStructDb::build(aws_sdk_dynamodb::Region::new("eu-west-1"), "exampleTable").await;
    }
}
