use std::thread::sleep;
use std::time::Duration;
use aws_sdk_dynamodb::config::Region;
use dynamodb_helper::DynamoDb;

#[derive(DynamoDb, Debug)]
#[exclusion("batch_get")]
pub struct ExampleStruct {
    #[partition]
    partition_key: String,
    a_number: u32,
    an_optional_string: Option<String>,
}

#[tokio::main]
async fn main() {
    // unlike the tests, this example uses real DynamoDB - so you'll need credentials if you want this to work //
    println!("Setting up our client");
    let client = ExampleStructDb::build(Region::new("eu-west-1"), "dynamoDbHelperExampleTable").await;
    
    println!("Creating table");
    client.create_table().await.expect("Create table to work");
    println!("Waiting a bit until table is available");
    sleep(Duration::from_secs(15)); // better to check readiness, but ok for demonstration purposes

    let example = ExampleStruct {
        partition_key: "abc123".to_string(),
        a_number: 5,
        an_optional_string: Some("optional value".to_string()),
    };

    println!("Putting an example in the table");
    client.put(example).await.expect("To be able to put our struct in the table");
    sleep(Duration::from_secs(1)); // if we're too fast, the item might not be there yet

    println!("Retrieving the example by its partition key");
    let result = client.get("abc123".to_string()).await.expect("To be able to get our struct back");
    println!("Got back {result:?}");

    println!("Cleaning up");
    client.delete_table().await.expect("Delete table to work");
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use super::*;

    #[derive(DynamoDb)]
    #[exclusion("new", "get", "batch_get", "put", "batch_put", "delete", "scan", "create_table", "delete_table")]
    pub struct PrettyUselessTestStruct {
        #[partition]
        partition_key: String,
        value: String,
    }

    #[derive(DynamoDb)]
    #[exclusion("delete_table", "create_table")]
    pub struct ExampleTestStruct {
        #[partition]
        partition_key: String,
        value: i32,
        true_or_false: bool,
        some_string: String,
        number_liz: Vec<i16>,
        string_list: Vec<String>,
        a_map: HashMap<String, String>,
        optional_string: Option<String>,
        optional_number: Option<i32>,
        optional_number_list: Option<Vec<f32>>,
        optional_number_map: Option<HashMap<String, String>>,
    }

    #[tokio::test]
    async fn should_generate_a_helper_struct_with_build_method() {
        let _e = ExampleTestStruct {
            partition_key: "example".to_string(),
            value: 0,
            true_or_false: false,
            some_string: "".to_string(),
            number_liz: vec![],
            string_list: vec![],
            a_map: Default::default(),
            optional_string: None,
            optional_number: None,
            optional_number_list: None,
            optional_number_map: None
        };
        let _help = ExampleTestStructDb::build(Region::new("eu-west-1"), "exampleTable").await;
    }

    #[tokio::test]
    async fn should_generate_a_pretty_useless_struct_with_only_a_build_method() {
        let _e = PrettyUselessTestStruct {
            partition_key: "example".to_string(),
            value: "".to_string(),
        };
        let _help = PrettyUselessTestStructDb::build(Region::new("eu-west-1"), "exampleTable").await;
    }
}
