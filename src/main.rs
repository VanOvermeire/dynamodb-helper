use std::collections::HashMap;
use aws_config::SdkConfig;
use aws_sdk_dynamodb::Client;
use aws_sdk_dynamodb::error::{BatchGetItemError, GetItemError, PutItemError};
use aws_sdk_dynamodb::model::AttributeValue;
use aws_sdk_dynamodb::output::{GetItemOutput, PutItemOutput};
use aws_sdk_dynamodb::types::SdkError;
use dynamodb_helper::DynamoDb;

// TODO remove
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

    async fn build(region: aws_sdk_dynamodb::Region) -> Self {
        let region_provider = aws_config::meta::region::RegionProviderChain::first_try(region).or_default_provider();
        let shared_config = aws_config::from_env().region(region_provider).load().await;
        TestDB::new(Client::new(&shared_config), "".to_string())
    }

    pub async fn put(&self) -> Result<PutItemOutput, SdkError<PutItemError>> {
        let mut second = std::collections::HashMap::from([
            ("Mercury".to_string(), AttributeValue::N("0.4".to_string())),
            ("Venus".to_string(), AttributeValue::N("0.7".to_string()))
        ]);

        self.client.put_item()
            .table_name(&self.table)
            .set_item(Some(second))
            .send()
            .await
    }

    pub async fn get(&self, key: String) -> Result<Option<HashMap<String, AttributeValue>>, SdkError<GetItemError>> {
        let res = self.client.get_item()
            .table_name(&self.table)
            .key("partition_key", AttributeValue::S(key))
            .send()
            .await?;


        Ok(res.item)
    }

    // pub async fn batch_get(&self, keys: Vec<String>) -> Result<Option<&HashMap<String, Vec<HashMap<String, AttributeValue>>>>, aws_sdk_dynamodb::error::BatchGetItemError> {
    //     let res = self.client.batch_get_item()
    //         .set_request_items()
    //         .send()
    //         .await?;
    //     Ok(res.responses())
    // }
}

impl From<TestStruct> for HashMap<String, AttributeValue> {
    fn from(_: TestStruct) -> Self {
        todo!()
    }
}

impl From<HashMap<String, AttributeValue>> for TestStruct {
    fn from(map: HashMap<String, AttributeValue>) -> Self {
        let partition_key = map.get("partition_key").map(|v| v.as_s().expect("Conversion from AttributeValue to String to work")).expect("Value to be present");
        let value = map.get("value").map(|v| v.as_n().expect("Conversion from AttributeValue to String to work")).map(|v| str::parse(v).expect("To be able to parse a number from Dynamo")).expect("Value to be present");

        TestStruct {
            partition_key: partition_key.to_string(),
            value
        }
    }
}

#[tokio::main]
async fn main() {}

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
