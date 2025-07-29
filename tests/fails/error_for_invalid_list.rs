use dynamodb_helper::DynamoDb;

#[derive(DynamoDb)]
pub struct Example {
    #[partition]
    first: String,
    second: u32,
    invalid: Vec<bool>,
}

fn main() {}
