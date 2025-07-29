use dynamodb_helper::DynamoDb;

#[derive(DynamoDb)]
pub struct Example {
    #[partition]
    invalid: Vec<String>,
    first: String,
    second: u32,
}

fn main() {}
