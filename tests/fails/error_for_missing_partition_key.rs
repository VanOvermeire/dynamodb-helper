use dynamodb_helper::DynamoDb;

#[derive(DynamoDb)]
pub struct Example {
    first: String,
    second: u32,
}

fn main() {}
