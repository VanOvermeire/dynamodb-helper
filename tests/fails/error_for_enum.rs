use dynamodb_helper::DynamoDb;

#[derive(DynamoDb)]
pub enum SomeEnum {
    Nested(String)
}

fn main() {}
