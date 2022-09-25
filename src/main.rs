use aws_sdk_dynamodb::error::PutItemError;
use aws_sdk_dynamodb::output::PutItemOutput;
use aws_sdk_dynamodb::types::SdkError;
use dynamodb_helper::DynamoDb;

// use std::rc::Rc;
// use aws_config::meta::region::RegionProviderChain;
// use aws_sdk_dynamodb::{Client, Region};
// use crate::config;
//
// pub const DB_ID_NAME: &'static str = "id";
// pub const DB_EMAIL_NAME: &'static str = "email";
// pub const DB_CHARGER_ID: &'static str = "charger_id";

// use std::time::{SystemTime, UNIX_EPOCH};
// use async_trait::async_trait;
// use aws_sdk_dynamodb::model::AttributeValue;
// use common::{DB_ID_NAME, DbClient, NorthEastLatitude, NorthEastLongitude, SouthWestLatitude, SouthWestLongitude, Coordinate, Email, DB_EMAIL_NAME, ChargerId, DB_CHARGER_ID};
// use crate::adapters::AdapterError;

// #[async_trait]
// impl CoordinatesDb for DbClient {
//     async fn add(&self,
//                  table: &str, email: Email, charger_id: ChargerId,
//                  ne_lat: NorthEastLatitude, ne_lon: NorthEastLongitude,
//                  sw_lat: SouthWestLatitude, sw_lon: SouthWestLongitude) -> Result<(), AdapterError> {
//         let id = generate_id();
//
//         match &self.get_client_ref().put_item()
//             .table_name(table)
//             .item(DB_ID_NAME, AttributeValue::S(id))
//             .item(DB_EMAIL_NAME, AttributeValue::S(email.0))
//             .item(DB_CHARGER_ID, AttributeValue::N(charger_id.0.to_string()))
//             .item(ne_lon.get_name(), ne_lon.into())
//             .item(ne_lat.get_name(), ne_lat.into())
//             .item(sw_lat.get_name(), sw_lat.into())
//             .item(sw_lon.get_name(), sw_lon.into())
//             .send()
//             .await {
//             Ok(_) => Ok(()),
//             Err(e) => {
//                 println!("Error from database: {:?}", e);
//                 Err(AdapterError::DatabaseError)
//             }
//         }
//     }
// }

// TODO remove
struct TestStruct {
    partition_key: String,
    value: i32,
}

struct TestDB {
    client: aws_sdk_dynamodb::Client
}

impl TestDB {
    fn new(client: aws_sdk_dynamodb::Client) -> Self {
        TestDB {
            client,
        }
    }

    async fn build(region: aws_sdk_dynamodb::Region) -> Self {
        let region_provider = aws_config::meta::region::RegionProviderChain::first_try(region).or_default_provider();
        let shared_config = aws_config::from_env().region(region_provider).load().await;
        TestDB {
            client: aws_sdk_dynamodb::Client::new(&shared_config)
        }
    }

    pub async fn put(&self) -> Result<PutItemOutput, SdkError<PutItemError>> {
        let mut second = std::collections::HashMap::from([
            ("Mercury".to_string(), aws_sdk_dynamodb::model::AttributeValue::N("0.4".to_string())),
            ("Venus".to_string(), aws_sdk_dynamodb::model::AttributeValue::N("0.7".to_string()))
        ]);

        self.client.put_item()
            .table_name("")
            .set_item(Some(second))
            .send()
            .await
    }
}

impl Into<std::collections::HashMap<String, aws_sdk_dynamodb::model::AttributeValue>> for TestStruct {
    fn into(self) -> std::collections::HashMap<String, aws_sdk_dynamodb::model::AttributeValue> {
        let mut map = std::collections::HashMap::new();
        map.insert("partition_key".to_string(), aws_sdk_dynamodb::model::AttributeValue::S(self.partition_key));
        map.insert("value".to_string(), aws_sdk_dynamodb::model::AttributeValue::N(self.value.to_string()));
        map
    }
}

#[tokio::main]
async fn main() {
    // #[derive(DynamoDb)]
    // #[table = "exampleTable"]
    // struct ExampleStruct {
    //     partition_key: String,
    // }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use super::*;

    #[derive(DynamoDb)]
    #[table = "exampleTable"]
    struct ExampleStruct {
        partition_key: String,
        // value: i32,
    }

    #[tokio::test]
    async fn should_generate_a_helper_struct() {
        let e = ExampleStruct { partition_key: "example".to_string() };
        let help = ExampleStructDb::build(aws_sdk_dynamodb::Region::new("eu-west-1")).await;
        help.put(e);
    }
}
