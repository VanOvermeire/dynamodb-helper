mod util;
use util::*;

#[tokio::test]
async fn should_be_able_to_create_a_table() {
    let create_table = "createTableTable";
    let client = create_client().await;
    let client_for_struct = create_client().await;

    let db = OrderStructWithRangeDb::new(client_for_struct, create_table);

    db.create_table().await.expect("Create table to work");

    let results = client.list_tables()
        .send()
        .await
        .expect("To be able to list table");

    assert!(results.table_names.is_some() && results.table_names().unwrap().contains(&create_table.to_string()));

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
