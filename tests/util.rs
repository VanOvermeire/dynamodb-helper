use std::collections::HashMap;
use aws_sdk_dynamodb::{Client, Endpoint};
use aws_sdk_dynamodb::model::{AttributeDefinition, AttributeValue, BillingMode, KeySchemaElement, KeyType, ProvisionedThroughput, ScalarAttributeType};
use http::Uri;
use dynamodb_helper::DynamoDb;
use tokio_stream::StreamExt;

#[derive(DynamoDb, Debug)]
pub struct OrderStruct {
    #[partition]
    pub an_id: String,
    pub name: String,
    pub total_amount: f32,
}

#[derive(DynamoDb, Debug)]
pub struct OrderStructWithRange {
    #[partition]
    pub an_id: String,
    #[range]
    pub a_range: i32,
    pub name: String,
    pub total_amount: i32,
}

pub async fn create_client() -> Client {
    let config = aws_config::load_from_env().await;
    let dynamodb_local_config = aws_sdk_dynamodb::config::Builder::from(&config)
        .endpoint_resolver(Endpoint::immutable(Uri::from_static("http://localhost:8000")))
        .build();
    let client = Client::from_conf(dynamodb_local_config);
    client
}

pub async fn init_table(client: &Client, table_name: &str, partition_key: &str, range_key_option: Option<&str>) {
    let ads = if let Some(range_key) = range_key_option {
        vec![
            AttributeDefinition::builder()
                .attribute_name(partition_key)
                .attribute_type(ScalarAttributeType::S)
                .build(),
            AttributeDefinition::builder()
                .attribute_name(range_key)
                .attribute_type(ScalarAttributeType::N)
                .build(),
        ]
    } else {
        vec![
            AttributeDefinition::builder()
                .attribute_name(partition_key)
                .attribute_type(ScalarAttributeType::S)
                .build()
        ]
    };

    let keys = if let Some(range_key) = range_key_option {
        vec![
            KeySchemaElement::builder()
                .key_type(KeyType::Hash)
                .attribute_name(partition_key)
                .build(),
            KeySchemaElement::builder()
                .key_type(KeyType::Range)
                .attribute_name(range_key)
                .build(),
        ]
    } else {
        vec![
            KeySchemaElement::builder()
                .key_type(KeyType::Hash)
                .attribute_name(partition_key)
                .build()
        ]
    };

    client.create_table()
        .table_name(table_name)
        .set_key_schema(Some(keys))
        .set_attribute_definitions(Some(ads))
        .billing_mode(BillingMode::PayPerRequest)
        .send()
        .await
        .expect("Creating a table to work");
}

pub async fn destroy_table(client: &Client, table_name: &str) {
    client.delete_table()
        .table_name(table_name)
        .send()
        .await
        .expect("Deleting a table to work");
}

pub async fn put_order_struct(table: &str, client: &Client, struc: OrderStruct) {
    client.put_item()
        .table_name(table)
        .set_item(Some(HashMap::from([
            ("an_id".to_string(), AttributeValue::S(struc.an_id)),
            ("name".to_string(), AttributeValue::S(struc.name)),
            ("total_amount".to_string(), AttributeValue::N(struc.total_amount.to_string())),
        ])))
        .send()
        .await
        .expect("To be able to put");
}

pub async fn put_order_with_range_struct(table: &str, client: &Client, example: &OrderStructWithRange) {
    client.put_item()
        .table_name(table)
        .set_item(Some(HashMap::from([
            ("an_id".to_string(), AttributeValue::S(example.an_id.to_string())),
            ("a_range".to_string(), AttributeValue::N(example.a_range.to_string())),
            ("name".to_string(), AttributeValue::S(example.name.to_string())),
            ("total_amount".to_string(), AttributeValue::N(example.total_amount.to_string())),
        ])))
        .send()
        .await
        .expect("To be able to put");
}
