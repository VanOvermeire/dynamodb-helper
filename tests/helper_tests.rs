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
    // total_amount: f32,
}

#[tokio::test]
async fn should_be_able_to_get_from_dynamo() {
    let get_table = "getTable";
    let client = create_client().await;
    let client_for_struct = create_client().await;

    init_table(&client, get_table).await;

    let example = OrderStruct {
        an_id: "uid1234".to_string(),
        name: "Me".to_string(),
    };

    let db = OrderStructDb::new(client_for_struct, get_table);

    client.put_item()
        .table_name(get_table)
        .set_item(Some(HashMap::from([
            ("an_id".to_string(), AttributeValue::S(example.an_id)),
            ("name".to_string(), AttributeValue::S(example.name)),
        ])))
        .send()
        .await
        .expect("To be able to put");

    let result = db.get("uid1234".to_string())
        .await
        .expect("To be able to get a result");

    assert_eq!(result.an_id, "uid1234");
    assert_eq!(result.name, "Me");

    destroy_table(&client, get_table).await;
}

#[tokio::test]
async fn should_be_able_to_put_in_dynamo() {
    let put_table = "putTable";
    let client = create_client().await;
    let client_for_struct = create_client().await;

    init_table(&client, put_table).await;

    let example = OrderStruct {
        an_id: "uid123".to_string(),
        name: "Me".to_string(),
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

    println!("{:?}", result);

    assert!(result.item().is_some());

    destroy_table(&client, put_table).await;
}

async fn create_client() -> Client {
    let config = aws_config::load_from_env().await;
    let dynamodb_local_config = aws_sdk_dynamodb::config::Builder::from(&config)
        .endpoint_resolver(Endpoint::immutable(Uri::from_static("http://localhost:8000")))
        .build();
    let client = Client::from_conf(dynamodb_local_config);
    client
}

async fn init_table(client: &Client, table_name: &str) {
    let ad = AttributeDefinition::builder()
        .attribute_name("an_id")
        .attribute_type(ScalarAttributeType::S)
        .build();

    let key = KeySchemaElement::builder()
        .key_type(KeyType::Hash)
        .attribute_name("an_id")
        .build();

    let pt = ProvisionedThroughput::builder()
        .read_capacity_units(10)
        .write_capacity_units(5)
        .build();

    let _ = client.create_table()
        .table_name(table_name)
        .key_schema(key)
        .attribute_definitions(ad)
        .provisioned_throughput(pt)
        .send()
        .await;
}

async fn destroy_table(client: &Client, table_name: &str) {
    client.delete_table()
        .table_name(table_name)
        .send()
        .await
        .expect("Deleting table to work");
}
