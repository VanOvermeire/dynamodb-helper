pub mod util;
use util::*;

#[tokio::test]
async fn should_be_able_to_put() {
    let put_table = "putTable";
    let client = create_client().await;
    let client_for_struct = create_client().await;
    let example = create_order_struct();

    init_table(&client, put_table, "an_id", None).await;

    let db = OrderStructDb::new(client_for_struct, put_table);

    db.put(example.clone()).await.expect("Put to work");

    let result = get_order_struct(put_table, &client, example.an_id.as_str()).await;

    destroy_table(&client, put_table).await;

    assert!(result.item().is_some());
}

#[tokio::test]
async fn should_be_able_to_put_with_range_key() {
    let put_table = "putRangeTable";
    let client = create_client().await;
    let client_for_struct = create_client().await;
    let example = create_order_struct_with_range();

    init_table(&client, put_table, "an_id", Some("a_range")).await;

    let db = OrderStructWithRangeDb::new(client_for_struct, put_table);

    db.put(example.clone()).await.expect("Put to work");

    let result = get_order_struct_with_range(put_table, &client, example.an_id.as_str(), &example.a_range).await;

    destroy_table(&client, put_table).await;

    assert!(result.item().is_some());
}

#[tokio::test]
async fn should_be_able_to_update() {
    let put_table = "updateTable";
    let client = create_client().await;
    let client_for_struct = create_client().await;
    let example = create_order_struct();

    init_table(&client, put_table, "an_id", None).await;

    put_order_struct(put_table, &client, &example).await;

    let db = OrderStructDb::new(client_for_struct, put_table);

    let mut updated = example.clone();
    updated.name = "Another name".to_string();

    db.put(updated).await.expect("Put to work");

    let result = get_order_struct(put_table, &client, example.an_id.as_str()).await;

    destroy_table(&client, put_table).await;

    assert!(result.item().is_some());
    assert_eq!(
        result.item().unwrap().get("name").unwrap().as_s().unwrap(),
        &"Another name".to_string()
    );
}

#[tokio::test]
async fn should_be_able_to_batch_put() {
    let put_table = "batchPutTable";
    let client = create_client().await;
    let client_for_struct = create_client().await;
    let example = create_order_struct();

    let db = OrderStructDb::new(client_for_struct, put_table);

    init_table(&client, put_table, "an_id", None).await;

    db.batch_put(vec![example.clone()]).await.expect("Batch put to work");

    let result = get_order_struct(put_table, &client, example.an_id.as_str()).await;

    destroy_table(&client, put_table).await;

    assert!(result.item().is_some());
}

#[tokio::test]
async fn should_be_able_to_batch_put_with_range() {
    let put_table = "batchPutRangeTable";
    let client = create_client().await;
    let client_for_struct = create_client().await;
    let example = OrderStructWithRange {
        an_id: "uid123".to_string(),
        a_range: 1000,
        name: "Me".to_string(),
        total_amount: 6,
        names: vec!["one".to_string()],
        map_values: Default::default(),
    };
    let second_example = OrderStructWithRange {
        an_id: "uid123".to_string(),
        a_range: 1001,
        name: "You".to_string(),
        total_amount: 7,
        names: vec!["two".to_string()],
        map_values: Default::default(),
    };

    init_table(&client, put_table, "an_id", Some("a_range")).await;

    let db = OrderStructWithRangeDb::new(client_for_struct, put_table);

    db.batch_put(vec![example.clone(), second_example.clone()])
        .await
        .expect("Batch put to work");

    let result = get_order_struct_with_range(put_table, &client, example.an_id.as_str(), &example.a_range).await;
    let second_result = get_order_struct_with_range(put_table, &client, second_example.an_id.as_str(), &second_example.a_range).await;

    destroy_table(&client, put_table).await;

    assert!(result.item().is_some());
    assert!(second_result.item().is_some());
}

#[tokio::test]
async fn should_be_able_to_delete() {
    let delete_table = "deleteTable";
    let client = create_client().await;
    let client_for_struct = create_client().await;
    let example = create_order_struct();

    init_table(&client, delete_table, "an_id", None).await;

    put_order_struct(delete_table, &client, &example).await;

    let db = OrderStructDb::new(client_for_struct, delete_table);

    db.delete(example.an_id.to_string()).await.expect("Delete to work");

    let result = get_order_struct(delete_table, &client, example.an_id.as_str()).await;

    destroy_table(&client, delete_table).await;

    assert!(result.item().is_none());
}

#[tokio::test]
async fn should_be_able_to_delete_with_range() {
    let delete_table = "deleteRangeTable";
    let client = create_client().await;
    let client_for_struct = create_client().await;
    let example = create_order_struct_with_range();

    init_table(&client, delete_table, "an_id", Some("a_range")).await;

    put_order_with_range_struct(delete_table, &client, &example).await;

    let db = OrderStructWithRangeDb::new(client_for_struct, delete_table);

    db.delete(example.an_id.to_string(), example.a_range).await.expect("Delete to work");

    let result = get_order_struct_with_range(delete_table, &client, example.an_id.as_str(), &example.a_range).await;

    destroy_table(&client, delete_table).await;

    assert!(result.item().is_none());
}
