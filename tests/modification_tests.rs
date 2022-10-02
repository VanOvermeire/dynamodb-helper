use std::collections::HashMap;
use aws_sdk_dynamodb::model::{AttributeValue};
use std::iter::Iterator;
use aws_sdk_dynamodb::Client;
use aws_sdk_dynamodb::output::GetItemOutput;
use aws_sdk_dynamodb::types::SdkError;
use http::Uri;
use tokio_stream::StreamExt;

mod util;
use util::*;

#[tokio::test]
async fn should_be_able_to_put() {
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

    let result = get_order_struct(put_table, &client, "uid123").await;

    destroy_table(&client, put_table).await;

    assert!(result.item().is_some());
}

#[tokio::test]
async fn should_be_able_to_put_with_range_key() {
    let put_table = "putRangeTable";
    let client = create_client().await;
    let client_for_struct = create_client().await;

    init_table(&client, put_table, "an_id", Some("a_range")).await;

    let example = OrderStructWithRange {
        an_id: "uid123".to_string(),
        a_range: 1000,
        name: "Me".to_string(),
        total_amount: 6,
    };

    let db = OrderStructWithRangeDb::new(client_for_struct, put_table);

    db.put(example)
        .await
        .expect("Put to work");

    let result = get_order_struct_with_range(put_table, &client, "uid123", 1000).await;

    destroy_table(&client, put_table).await;

    assert!(result.item().is_some());
}

#[tokio::test]
async fn should_be_able_to_delete() {
    let delete_table = "deleteTable";
    let client = create_client().await;
    let client_for_struct = create_client().await;

    init_table(&client, delete_table, "an_id", None).await;

    let example = OrderStruct {
        an_id: "uid123".to_string(),
        name: "Me".to_string(),
        total_amount: 6.0,
    };

    put_order_struct(delete_table, &client, &example);

    let db = OrderStructDb::new(client_for_struct, delete_table);

    db.delete(example.an_id.to_string()).await;

    let result = get_order_struct(delete_table, &client, "uid123").await;

    get_order_struct(delete_table, &client, "uid123");

    destroy_table(&client, delete_table).await;

    assert!(result.item().is_none());
}

#[tokio::test]
async fn should_be_able_to_delete_with_range() {
    let delete_table = "deleteRangeTable";
    let client = create_client().await;
    let client_for_struct = create_client().await;

    init_table(&client, delete_table, "an_id", Some("a_range")).await;

    let example = OrderStructWithRange {
        an_id: "uid123".to_string(),
        a_range: 1000,
        name: "Me".to_string(),
        total_amount: 6,
    };

    put_order_with_range_struct(delete_table, &client, &example);

    let db = OrderStructWithRangeDb::new(client_for_struct, delete_table);

    db.delete(example.an_id, example.a_range).await;

    let result = get_order_struct_with_range(delete_table, &client, "uid123", 1000).await;

    destroy_table(&client, delete_table).await;

    assert!(result.item().is_none());
}
