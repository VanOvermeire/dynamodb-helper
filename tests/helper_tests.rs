use std::any::Any;
use std::collections::HashMap;
use std::error;
use aws_config::SdkConfig;
use aws_sdk_dynamodb::{Client, Endpoint, Region};
use aws_sdk_dynamodb::model::{AttributeDefinition, AttributeValue, KeySchemaElement, KeyType, ProvisionedThroughput, ScalarAttributeType};
use aws_sdk_dynamodb::types::SdkError;
use dynamodb_helper::DynamoDb;
use http::Uri;

#[derive(DynamoDb, Debug)]
pub struct OrderStruct {
    #[partition]
    an_id: String,
    name: String,
    total_amount: f32,
}

#[derive(DynamoDb, Debug)]
pub struct OrderStructWithRange {
    #[partition]
    an_id: String,
    #[range]
    a_range: i32,
    name: String,
    total_amount: i32,
}

// TODO use a closure so we don't have to repeat all this setup?

#[tokio::test]
async fn should_be_able_to_get_from_dynamo() {
    let get_table = "getTable";
    let client = create_client().await;
    let client_for_struct = create_client().await;

    init_table(&client, get_table, "an_id", None).await;

    let example = OrderStruct {
        an_id: "uid1234".to_string(),
        name: "Me".to_string(),
        total_amount: 5.0,
    };

    let db = OrderStructDb::new(client_for_struct, get_table);

    client.put_item()
        .table_name(get_table)
        .set_item(Some(HashMap::from([
            ("an_id".to_string(), AttributeValue::S(example.an_id)),
            ("name".to_string(), AttributeValue::S(example.name)),
            ("total_amount".to_string(), AttributeValue::N(example.total_amount.to_string())),
        ])))
        .send()
        .await
        .expect("To be able to put");

    let result = db.get("uid1234".to_string())
        .await
        .expect("To be able to get a result");

    destroy_table(&client, get_table).await;

    assert_eq!(result.an_id, "uid1234");
    assert_eq!(result.name, "Me");
    assert_eq!(result.total_amount, 5.0);

}

#[tokio::test]
async fn should_be_able_to_get_from_dynamo_with_range_key() {
    let get_table = "getRangeTable";
    let client = create_client().await;
    let client_for_struct = create_client().await;

    init_table(&client, get_table, "an_id", Some("a_range")).await;

    let example = OrderStructWithRange {
        an_id: "uid123".to_string(),
        a_range: 1000,
        name: "Me".to_string(),
        total_amount: 6,
    };
    let db = OrderStructWithRangeDb::new(client_for_struct, get_table);

    client.put_item()
        .table_name(get_table)
        .set_item(Some(HashMap::from([
            ("an_id".to_string(), AttributeValue::S(example.an_id.to_string())),
            ("a_range".to_string(), AttributeValue::N(example.a_range.to_string())),
            ("name".to_string(), AttributeValue::S(example.name)),
            ("total_amount".to_string(), AttributeValue::N(example.total_amount.to_string())),
        ])))
        .send()
        .await
        .expect("To be able to put");

    let result = db.get(example.an_id.to_string(), example.a_range)
        .await
        .expect("To be able to get a result");

    destroy_table(&client, get_table).await;

    assert_eq!(result.an_id, "uid123");
    assert_eq!(result.a_range, 1000);
    assert_eq!(result.name, "Me");
    assert_eq!(result.total_amount, 6);

}

#[tokio::test]
async fn should_be_able_to_put_in_dynamo() {
    let put_table = "putTable";
    let client = create_client().await;
    let client_for_struct = create_client().await;

    init_table(&client, put_table, "an_id", None).await;

    let example = OrderStruct {
        an_id: "uid123".to_string(),
        name: "Me".to_string(),
        total_amount: 6.0,
    };

    let db = OrderStructDb::new(client_for_struct, put_table);

    db.put(example)
        .await
        .expect("Put to work");

    let result = client.get_item()
        .table_name(put_table)
        .key("an_id".to_string(), AttributeValue::S("uid123".to_string()))
        .send()
        .await
        .expect("To be able to get a result");

    destroy_table(&client, put_table).await;

    assert!(result.item().is_some());
}

async fn create_client() -> Client {
    let config = aws_config::load_from_env().await;
    let dynamodb_local_config = aws_sdk_dynamodb::config::Builder::from(&config)
        .endpoint_resolver(Endpoint::immutable(Uri::from_static("http://localhost:8000")))
        .build();
    let client = Client::from_conf(dynamodb_local_config);
    client
}

async fn init_table(client: &Client, table_name: &str, partition_key: &str, range_key_option: Option<&str>) {
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
        .provisioned_throughput(ProvisionedThroughput::builder()
            .read_capacity_units(10)
            .write_capacity_units(5)
            .build())
        .send()
        .await
        .expect("Creating a table to work");
}

async fn destroy_table(client: &Client, table_name: &str) {
    client.delete_table()
        .table_name(table_name)
        .send()
        .await
        .expect("Deleting a table to work");
}
