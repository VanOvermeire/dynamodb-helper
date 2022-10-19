use std::iter::Iterator;

mod util;
use util::*;

#[tokio::test]
async fn should_be_able_to_get_from_dynamo() {
    let get_table = "getTable";
    let client = create_client().await;
    let client_for_struct = create_client().await;
    let example = create_order_struct();

    init_table(&client, get_table, "an_id", None).await;

    let db = OrderStructDb::new(client_for_struct, get_table);

    put_order_struct(get_table, &client, &example).await;

    let result_option = db.get(example.an_id.to_string())
        .await
        .expect("To be able to get a result");

    destroy_table(&client, get_table).await;

    assert!(result_option.is_some());

    let result = result_option.unwrap();

    assert_eq!(result.an_id, example.an_id);
    assert_eq!(result.name, example.name);
    assert_eq!(result.total_amount, example.total_amount);
    assert_eq!(result.a_boolean, example.a_boolean);
    assert_eq!(result.numbers, example.numbers);
}

#[tokio::test]
async fn should_be_able_to_get_from_dynamo_with_range_key() {
    let get_table = "getRangeTable";
    let client = create_client().await;
    let client_for_struct = create_client().await;
    let example = create_order_struct_with_range();

    init_table(&client, get_table, "an_id", Some("a_range")).await;

    let db = OrderStructWithRangeDb::new(client_for_struct, get_table);

    put_order_with_range_struct(get_table, &client, &example).await;

    let result_option = db.get(example.an_id.to_string(), example.a_range)
        .await
        .expect("To be able to get a result");

    destroy_table(&client, get_table).await;

    assert!(result_option.is_some());

    let result = result_option.unwrap();

    assert_eq!(result.an_id, example.an_id);
    assert_eq!(result.a_range, example.a_range);
    assert_eq!(result.name, example.name);
    assert_eq!(result.total_amount, example.total_amount);
}

#[tokio::test]
async fn should_be_able_to_get_from_dynamo_only_using_partition_part() {
    let get_table = "getByPartitionKeyTable";
    let client = create_client().await;
    let client_for_struct = create_client().await;
    let example = OrderStructWithRange {
        an_id: "uid123".to_string(),
        a_range: 1000,
        name: "Me".to_string(),
        total_amount: 6,
        names: vec!["one".to_string()],
        map_values: Default::default()
    };
    let second_example = OrderStructWithRange {
        an_id: "uid123".to_string(),
        a_range: 1001,
        name: "You".to_string(),
        total_amount: 7,
        names: vec!["two".to_string()],
        map_values: Default::default()
    };

    init_table(&client, get_table, "an_id", Some("a_range")).await;

    let db = OrderStructWithRangeDb::new(client_for_struct, get_table);

    put_order_with_range_struct(get_table, &client, &example).await;
    put_order_with_range_struct(get_table, &client, &second_example).await;

    let result = db.get_by_partition_key(example.an_id.to_string())
        .await
        .expect("Get by partition key to succeed");

    destroy_table(&client, get_table).await;

    assert_eq!(result.len(), 2);
    assert_eq!(result[0].a_range, example.a_range);
    assert_eq!(result[1].a_range, second_example.a_range);
}

#[tokio::test]
async fn should_be_able_to_get_multiple_items() {
    let get_table = "batchGetTable";
    let client = create_client().await;
    let client_for_struct = create_client().await;
    let example = create_order_struct();

    init_table(&client, get_table, "an_id", None).await;

    put_order_struct(get_table, &client, &example).await;

    let db = OrderStructDb::new(client_for_struct, get_table);

    let result = db.batch_get(vec![example.an_id])
        .await
        .expect("Batch get to succeed");

    destroy_table(&client, get_table).await;

    assert_eq!(result.len(), 1);
}

#[tokio::test]
async fn should_be_able_to_get_multiple_items_with_range_key() {
    let get_table = "batchGetRangeTable";
    let client = create_client().await;
    let client_for_struct = create_client().await;
    let example = OrderStructWithRange {
        an_id: "uid123".to_string(),
        a_range: 1000,
        name: "Me".to_string(),
        total_amount: 6,
        names: vec!["one".to_string()],
        map_values: Default::default()
    };
    let second_example = OrderStructWithRange {
        an_id: "uid123".to_string(),
        a_range: 1001,
        name: "You".to_string(),
        total_amount: 7,
        names: vec!["two".to_string()],
        map_values: Default::default()
    };

    init_table(&client, get_table, "an_id", Some("a_range")).await;

    let db = OrderStructWithRangeDb::new(client_for_struct, get_table);

    put_order_with_range_struct(get_table, &client, &example).await;
    put_order_with_range_struct(get_table, &client, &second_example).await;

    let result = db.batch_get(vec![(example.an_id, example.a_range), (second_example.an_id, second_example.a_range)])
        .await
        .expect("Batch get to succeed");

    destroy_table(&client, get_table).await;

    assert_eq!(result.len(), 2);
    assert_eq!(result.iter().map(|v| v.name.to_string()).collect::<Vec<String>>(), vec!["Me", "You"]);
}

#[tokio::test]
async fn should_be_able_to_scan_dynamo() {
    let scan_table = "myScanTable";
    let client = create_client().await;
    let client_for_struct = create_client().await;
    let example = OrderStruct {
        an_id: "uid1234".to_string(),
        name: "Me".to_string(),
        total_amount: 5.0,
        a_boolean: true,
        numbers: vec![]
    };
    let second_example = OrderStruct {
        an_id: "uid1235".to_string(),
        name: "You".to_string(),
        total_amount: 7.5,
        a_boolean: false,
        numbers: vec![]
    };

    init_table(&client, scan_table, "an_id", None).await;

    let db = OrderStructDb::new(client_for_struct, scan_table);

    put_order_struct(scan_table, &client, &example).await;
    put_order_struct(scan_table, &client, &second_example).await;

    let result = db.scan()
        .await
        .expect("Scan to succeed");

    destroy_table(&client, scan_table).await;

    assert_eq!(result.len(), 2);
}
