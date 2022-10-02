use std::collections::HashMap;
use aws_sdk_dynamodb::model::{AttributeValue};
use std::iter::Iterator;
use aws_sdk_dynamodb::Client;
use aws_sdk_dynamodb::types::SdkError;
use http::Uri;
use tokio_stream::StreamExt;

mod util;
use util::*;

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

#[tokio::test]
async fn should_be_able_to_put_in_dynamo_with_range_key() {
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

    let result = client.get_item()
        .table_name(put_table)
        .key("an_id".to_string(), AttributeValue::S("uid123".to_string()))
        .key("a_range".to_string(), AttributeValue::N(1000.to_string()))
        .send()
        .await
        .expect("To be able to get a result");

    destroy_table(&client, put_table).await;

    assert!(result.item().is_some());
}
