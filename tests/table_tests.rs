pub mod util;

use aws_sdk_dynamodb::error::SdkError;
use aws_sdk_dynamodb::operation::create_table::CreateTableError;
use aws_sdk_dynamodb::types::KeyType;
use util::*;

#[tokio::test]
async fn should_be_able_to_create_a_table() {
    let create_table = "createTableTable";
    let client = create_client().await;
    let client_for_struct = create_client().await;

    let db = OrderStructWithRangeDb::new(client_for_struct, create_table);

    db.create_table().await.expect("Create table to work");

    let result = client.describe_table()
        .table_name(create_table)
        .send()
        .await
        .expect("To be able to describe tables");

    assert!(result.table.is_some());

    let table = result.table.unwrap();

    assert_eq!(table.key_schema.as_ref().unwrap()[0].attribute_name, "an_id");
    assert_eq!(table.key_schema.as_ref().unwrap()[0].key_type, KeyType::Hash);
    assert_eq!(table.key_schema.as_ref().unwrap()[1].attribute_name, "a_range");
    assert_eq!(table.key_schema.as_ref().unwrap()[1].key_type, KeyType::Range);

    destroy_table(&client, create_table).await;
}

#[tokio::test]
async fn should_return_error_result_when_creating_table_twice() {
    let create_table = "createTableTwiceTable";
    let client = create_client().await;
    let client_for_struct = create_client().await;

    let db = OrderStructWithRangeDb::new(client_for_struct, create_table);

    db.create_table().await.expect("Create table to work");

    let result: Result<_, SdkError<CreateTableError>> = db.create_table().await;

    assert!(result.is_err());

    destroy_table(&client, create_table).await;
}

#[tokio::test]
async fn should_be_able_to_create_provisioned_table() {
    let create_table = "createProvisionedTableTable";
    let client = create_client().await;
    let client_for_struct = create_client().await;

    let db = OrderStructWithRangeDb::new(client_for_struct, create_table);

    db.create_table_with_provisioned_throughput(5, 7).await.expect("Create table to work");

    let result = client.describe_table()
        .table_name(create_table)
        .send()
        .await
        .expect("To be able to describe tables");

    assert!(result.table.is_some());

    let table = result.table.unwrap();

    assert_eq!(table.key_schema.as_ref().unwrap()[0].attribute_name, "an_id");
    assert_eq!(table.key_schema.as_ref().unwrap()[0].key_type, KeyType::Hash);
    assert_eq!(table.key_schema.as_ref().unwrap()[1].attribute_name, "a_range");
    assert_eq!(table.key_schema.as_ref().unwrap()[1].key_type, KeyType::Range);
    assert_eq!(table.provisioned_throughput.as_ref().unwrap().read_capacity_units.unwrap(), 5);
    assert_eq!(table.provisioned_throughput.as_ref().unwrap().write_capacity_units.unwrap(), 7);

    destroy_table(&client, create_table).await;
}

#[tokio::test]
async fn should_be_able_to_delete_a_table() {
    let delete_table = "deleteTableTable";
    let client = create_client().await;
    let client_for_struct = create_client().await;

    init_table(&client, delete_table, "an_id", Some("a_range")).await;

    let db = OrderStructDb::new(client_for_struct, delete_table);

    db.delete_table().await.expect("Delete table to work");

    let results = client.list_tables()
        .send()
        .await
        .expect("To be able to list table");

    let filtered = results.table_names.filter(|t| t.iter().any(|tab| tab == &delete_table.to_string()));

    assert!(filtered.is_none() || filtered.unwrap().len() == 0);
}

#[tokio::test]
async fn should_returning_error_result_when_deleting_twice() {
    let delete_table = "deleteTableTwiceTable";
    let client = create_client().await;
    let client_for_struct = create_client().await;

    init_table(&client, delete_table, "an_id", Some("a_range")).await;

    let db = OrderStructDb::new(client_for_struct, delete_table);

    db.delete_table().await.expect("Delete table to work");

    let result = db.delete_table().await;

    assert!(result.is_err());
}


