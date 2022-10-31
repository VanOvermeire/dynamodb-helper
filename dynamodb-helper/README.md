# DynamoDB Helper

This crate provides a macro that will generate a struct for interacting with DynamoDB, without all the boilerplate that this normally entails.

This project is similar to the [dynomite-derive](https://crates.io/crates/dynomite-derive) crate. 
But dynomite derive is based on an unofficial Rust SDK called [rusoto](https://github.com/rusoto/rusoto) that has since gone into maintenance mode, whereas here the official SDK is used.

## Example Usage

```
use dynamodb_helper::DynamoDb;
use tokio_stream::StreamExt; // needed if the scan method is not excluded

#[derive(DynamoDb)]
struct ExampleStruct {
    #[partition]
    id: String,
    // other values
}

// thanks to the above derive, we can now create a db client
let db = ExampleStructDb::build(aws_sdk_dynamodb::Region::new("eu-west-1"), "exampleTable").await;

// the following will return an ExampleStruct if the id is found
let example_struct = db.get("someId".to_string()).await.expect("This one to exist");

#[derive(DynamoDb)]
struct OtherStruct {
    #[partition]
    id: String,
    #[range]
    range_id: String
    // other values
}

// alternative to build, we can use new and pass in a client
let other_db = OtherStructDb::new(a_dynamodb_client, "exampleTable");

// now we need to pass in both parts of the id
let other_struct = other_db.get("someId".to_string(), "someRange".to_string()).await.expect("This one to exist");
// or only the partition id, in which case we'll get back a Vec
let multiple_structs = other_db.get_by_partition_key("someId".to_string()).await.expect("This one to exist");

// and you can also put, delete, scan, etc.
```

Also see the unit and integration tests.

Be sure to check the [usage info](#usage-notes) and the overview of [attributes and methods](#macro-details).

## Usage notes

### Dependencies

These dependencies are required:

```
aws-config = "0.47.0"
aws-sdk-dynamodb = "0.17.0"
```

And possibly the `tokio-stream` dependency as well (see below)

### StreamExt trait

If you get an error warning about `the following trait bounds were not satisfied: impl futures_core::stream::Stream<Item = Result<...> + Unpin: Iterator` this 
is probably caused by your use of the generated scan method, which requires the following trait to be in scope (add this to your file imports):

```
use tokio_stream::StreamExt;
```

And add the `tokio-stream` dependency. Alternatively, you can [exclude generation of the scan method](#exclusions).

## Macro details

### Macro attributes

- `#[partition]` should decorate the field that will serve as the partition/hash key
- `#[range]` can *optionally* be placed on a field that serves as a range/sort key

### Generated structs and methods

The macro will implement the following traits:
- `From<HashMap<String, AttributeValue>>` for your struct
- `From<YourStruct>` for `HashMap<String, AttributeValue>`

The macro will also generate a struct with the name of the annotated struct plus the suffix `Db`, with the following methods:
- `new()`
- `build()`
- `create_table()`
- `create_table_with_provisioned_throughput()`
- `delete_table()`
- `get()`
- `get_by_partition_key()` but only when you have a complex key (partition plus range)
- `batch_get()`
- `batch_put()` which only works for *new* items
- `scan()`
- `put()`
- `delete()`

The create and delete table methods are appropriate for testing, pocs and smaller projects. For real applications it is probably better to create the tables as IAC and to pass the names to `new()` or `build()`.

Both the client and table name are exposed as public fields in case you also want to use these fields for custom queries.

### Supported types

Within your struct you can use the following types:
- Numbers
- Strings
- Booleans
- `Vec<String>`
- `Vec<Number>`
- `HashMap<String, String>`

Note that DynamoDB only supports strings, numbers and booleans *for key types*.

Saving as *string sets* or *number sets* is not possible, other types of maps and vecs are still TODO.

### Errors

Most methods return a result, with the error being the appropriate AWS error. For example, create_table returns `Result<aws_sdk_dynamodb::output::CreateTableOutput, aws_sdk_dynamodb::types::SdkError<aws_sdk_dynamodb::error::CreateTableError>>`.

Retrieval methods (gets, batch gets and scans) return a *custom error* because parsing the return value might fail. The returned error is an enum of the parse error and the aws error. The name of the error is based on the name of the struct.

For example, for the struct `ExampleStruct` our macro generates: 

```
pub enum ExampleStructDbScanError {
    ParseError(String),
    AwsError(aws_sdk_dynamodb::types::SdkError<aws_sdk_dynamodb::error::ScanError>),
}
```

And the scan method returns `Result<Vec<ExampleStruct>, ExampleStructDbScanError>`.

### Exclusions

You can optionally decide against generating methods. There are various reasons for doing this:
- `scan` requires an additional import and dependency
- some codes might be too dangerous to expose (like `delete_table`)
- more exclusions means less generated code. For example, with only build and get left enabled, the amount of generated code will be about almost 50% smaller (15 kb)

```
#[derive(DynamoDb)]
#[exclusion("scan", "delete_table", "create_table")]
pub struct ExampleTestStruct {
    #[partition]
    partition_key: String,
    value: i32,
}
```

'Exclusions' accepts the following parameters: "new", "build", "get" (which will also exclude get_by_partition_key when that's applicable), "batch_get", "put", "batch_put", "delete", "scan", "create_table" and "delete_table".

Traits and errors will only be generated when they necessary.
