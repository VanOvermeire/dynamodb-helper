use std::collections::HashMap;
use aws_config::SdkConfig;
use aws_sdk_dynamodb::Client;
use aws_sdk_dynamodb::error::{GetItemError, PutItemError};
use aws_sdk_dynamodb::model::AttributeValue;
use aws_sdk_dynamodb::output::{GetItemOutput, PutItemOutput};
use aws_sdk_dynamodb::types::SdkError;
use dynamodb_helper::DynamoDb;

// TODO remove
struct TestStruct {
    partition_key: String,
    // value: i32,
}

struct TestDB {
    client: aws_sdk_dynamodb::Client,
    table: String,
}

impl TestDB {
    fn new(client: aws_sdk_dynamodb::Client, table: String) -> Self {
        TestDB {
            client,
            table,
        }
    }

    async fn build(region: aws_sdk_dynamodb::Region) -> Self {
        let region_provider = aws_config::meta::region::RegionProviderChain::first_try(region).or_default_provider();
        let shared_config = aws_config::from_env().region(region_provider).load().await;
        TestDB::new(aws_sdk_dynamodb::Client::new(&shared_config), "".to_string())
    }

    pub async fn put(&self) -> Result<PutItemOutput, SdkError<PutItemError>> {
        let mut second = std::collections::HashMap::from([
            ("Mercury".to_string(), aws_sdk_dynamodb::model::AttributeValue::N("0.4".to_string())),
            ("Venus".to_string(), aws_sdk_dynamodb::model::AttributeValue::N("0.7".to_string()))
        ]);

        self.client.put_item()
            .table_name(&self.table)
            .set_item(Some(second))
            .send()
            .await
    }

    pub async fn get(&self, key: String) -> Result<Option<HashMap<String, AttributeValue>>, aws_sdk_dynamodb::types::SdkError<aws_sdk_dynamodb::error::GetItemError>> {
        let res = self.client.get_item()
            .table_name(&self.table)
            .key("partition_key", AttributeValue::S(key))
            .send()
            .await?;
        Ok(res.item)
    }
}

// impl Into<HashMap<String, AttributeValue>> for TestStruct {
//     fn into(self) -> HashMap<String, AttributeValue> {
//         let mut map = HashMap::new();
//         map.insert("partition_key".to_string(), AttributeValue::S(self.partition_key));
//         // map.insert("value".to_string(), aws_sdk_dynamodb::model::AttributeValue::N(self.value.to_string()));
//         map
//     }
// }

impl From<TestStruct> for HashMap<String, AttributeValue> {
    fn from(_: TestStruct) -> Self {
        todo!()
    }
}

impl From<HashMap<String, AttributeValue>> for TestStruct {
    fn from(map: HashMap<String, AttributeValue>) -> Self {
        let partition_key = map.get("partition_key").map(|v| v.as_s().expect("This to work")).expect("Value to be present");

        TestStruct {
            partition_key: partition_key.to_string(),
        }
    }
}

#[tokio::main]
async fn main() {
    #[derive(DynamoDb)]
    struct ExampleStruct {
        #[partition]
        partition_key: String,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use super::*;

    #[derive(DynamoDb)]
    struct ExampleTestStruct {
        #[partition]
        partition_key: String,
        // value: i32, TODO
    }

    #[tokio::test]
    async fn should_generate_a_helper_struct_with_build_method() {
        let e = ExampleTestStruct { partition_key: "example".to_string() };
        let help = ExampleTestStruct::build(aws_sdk_dynamodb::Region::new("eu-west-1"), "exampleTable").await;
    }
}
