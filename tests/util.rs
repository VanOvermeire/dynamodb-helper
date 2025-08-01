use aws_sdk_dynamodb::config::{Credentials, Region};
use aws_sdk_dynamodb::operation::get_item::GetItemOutput;
use aws_sdk_dynamodb::types::{AttributeDefinition, AttributeValue, BillingMode, KeySchemaElement, KeyType, ScalarAttributeType};
use aws_sdk_dynamodb::Client;
use dynamodb_helper::DynamoDb;
use std::collections::HashMap;

#[derive(DynamoDb, Debug, Clone)]
pub struct OrderStruct {
    #[partition]
    pub an_id: String,
    pub name: String,
    pub total_amount: f32,
    pub a_boolean: bool,
    pub numbers: Vec<i16>,
    pub something_optional: Option<String>,
}

#[derive(DynamoDb, Debug, Clone)]
pub struct OrderStructWithRange {
    #[partition]
    pub an_id: String,
    #[range]
    pub a_range: i32,
    pub name: String,
    pub total_amount: i32,
    pub names: Vec<String>,
    pub map_values: HashMap<String, String>,
}

pub fn create_order_struct() -> OrderStruct {
    OrderStruct {
        an_id: "uid123".to_string(),
        name: "Me".to_string(),
        total_amount: 6.0,
        a_boolean: false,
        numbers: vec![1, 2],
        something_optional: Some("something".to_string()),
    }
}

pub fn create_order_struct_with_range() -> OrderStructWithRange {
    OrderStructWithRange {
        an_id: "uid123".to_string(),
        a_range: 1000,
        name: "Me".to_string(),
        total_amount: 6,
        names: vec!["a name".to_string()],
        map_values: HashMap::from([("example".to_string(), "value".to_string())]),
    }
}

pub async fn create_client() -> Client {
    let config = aws_config::from_env()
        .region(Region::new("eu-central-1"))
        .credentials_provider(Credentials::new("accesskey", "privatekey", None, None, "dummy"))
        .load()
        .await;

    let dynamodb_local_config = aws_sdk_dynamodb::config::Builder::from(&config)
        .region(Some(Region::from_static("eu-west-1")))
        .endpoint_url("http://localhost:8000")
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
                .build()
                .unwrap(),
            AttributeDefinition::builder()
                .attribute_name(range_key)
                .attribute_type(ScalarAttributeType::N)
                .build()
                .unwrap(),
        ]
    } else {
        vec![AttributeDefinition::builder()
            .attribute_name(partition_key)
            .attribute_type(ScalarAttributeType::S)
            .build()
            .unwrap()]
    };

    let keys = if let Some(range_key) = range_key_option {
        vec![
            KeySchemaElement::builder()
                .key_type(KeyType::Hash)
                .attribute_name(partition_key)
                .build()
                .unwrap(),
            KeySchemaElement::builder()
                .key_type(KeyType::Range)
                .attribute_name(range_key)
                .build()
                .unwrap(),
        ]
    } else {
        vec![KeySchemaElement::builder()
            .key_type(KeyType::Hash)
            .attribute_name(partition_key)
            .build()
            .unwrap()]
    };

    client
        .create_table()
        .table_name(table_name)
        .set_key_schema(Some(keys))
        .set_attribute_definitions(Some(ads))
        .billing_mode(BillingMode::PayPerRequest)
        .send()
        .await
        .expect("Creating a table to work");
}

pub async fn destroy_table(client: &Client, table_name: &str) {
    client
        .delete_table()
        .table_name(table_name)
        .send()
        .await
        .expect("Deleting a table to work");
}

pub async fn put_order_struct(table: &str, client: &Client, struc: &OrderStruct) {
    let mut basic_map = HashMap::from([
        ("an_id".to_string(), AttributeValue::S(struc.an_id.to_string())),
        ("name".to_string(), AttributeValue::S(struc.name.to_string())),
        ("total_amount".to_string(), AttributeValue::N(struc.total_amount.to_string())),
        ("a_boolean".to_string(), AttributeValue::Bool(struc.a_boolean)),
        (
            "numbers".to_string(),
            AttributeValue::L(struc.numbers.iter().map(|v| AttributeValue::N(v.to_string())).collect()),
        ),
    ]);
    if struc.something_optional.is_some() {
        basic_map.insert(
            "something_optional".to_string(),
            AttributeValue::S(struc.something_optional.as_ref().unwrap().to_string()),
        );
    };
    put_hashmap(table, client, basic_map).await;
}

pub async fn put_order_with_range_struct(table: &str, client: &Client, example: &OrderStructWithRange) {
    let map = HashMap::from([
        ("an_id".to_string(), AttributeValue::S(example.an_id.to_string())),
        ("a_range".to_string(), AttributeValue::N(example.a_range.to_string())),
        ("name".to_string(), AttributeValue::S(example.name.to_string())),
        ("total_amount".to_string(), AttributeValue::N(example.total_amount.to_string())),
        (
            "names".to_string(),
            AttributeValue::L(example.names.iter().map(|v| v.clone()).map(|v| AttributeValue::S(v)).collect()),
        ),
        (
            "map_values".to_string(),
            AttributeValue::M(
                example
                    .map_values
                    .iter()
                    .map(|v| (v.0.clone(), AttributeValue::S(v.1.clone())))
                    .collect(),
            ),
        ),
    ]);
    put_hashmap(table, client, map).await;
}

pub async fn put_hashmap(table: &str, client: &Client, map: HashMap<String, AttributeValue>) {
    client
        .put_item()
        .table_name(table)
        .set_item(Some(map))
        .send()
        .await
        .expect("To be able to put");
}

pub async fn get_order_struct(table: &str, client: &Client, id: &str) -> GetItemOutput {
    client
        .get_item()
        .table_name(table)
        .key("an_id".to_string(), AttributeValue::S(id.to_string()))
        .send()
        .await
        .expect("To be able to get a result")
}

pub async fn get_order_struct_with_range(table: &str, client: &Client, id: &str, range: &i32) -> GetItemOutput {
    client
        .get_item()
        .table_name(table)
        .key("an_id".to_string(), AttributeValue::S(id.to_string()))
        .key("a_range".to_string(), AttributeValue::N(range.to_string()))
        .send()
        .await
        .expect("To be able to get a result")
}
