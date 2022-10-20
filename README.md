# DynamoDB Helper

This crate provides a macro that will generate a struct for interacting with DynamoDB, without all the boilerplate that this normally entails.

This project is similar to the [dynomite-derive](https://crates.io/crates/dynomite-derive) crate. 
But dynomite derive is based on an unofficial Rust SDK called [rusoto](https://github.com/rusoto/rusoto) that has since gone into maintenance mode, whereas here the official SDK is used.

## Status

![Github Actions Status](https://github.com/VanOvermeire/dynamodb-helper/actions/workflows/github-deploy.yml/badge.svg)

## Example Usage

```
use dynamodb_helper::DynamoDb;

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

And add the `tokio-stream` dependency. Alternatively, you can [exclude scan](#exclusions).

## Macro details

### Macro attributes

- `#[partition]` should decorate the field that will serve as the partition/hash key
- `#[range]` can optionally be placed on a field that serves as a range/sort key

### Generated structs and methods

The macro will implement the following traits:
- `From<HashMap<String, AttributeValue>>` for your struct
- `From<YourStruct>` for `HashMap<String, AttributeValue>`

The macro will generate a new struct, with the name of the annotated struct plus the suffix `Db`.

It has the following methods:
- `new()`
- `build()`
- `createTable()`
- `deleteTable()`
- `get()`
- `get_by_partition_key()` when you have a complex key (partition plus range)
- `batch_get()`
- `batch_put()` which only works for *new* items
- `scan()`
- `put()`
- `delete()`

The create and delete table methods are appropriate for testing, pocs and smaller projects. For real applications it is probably better to create the tables as IAC and to pass the names to `new()` or `build()`.

Both the client and table name are exposed as public fields in case you want to use the helper for custom queries.

### Supported types

Within your struct you can use the following types for now:
- Numbers
- Strings
- Booleans
- Vec<String>
- Vec<Number>
- HashMap<String, String>

Do note that DynamoDB only supports strings, numbers and booleans for key types.

Saving as string or number *sets* is not possible.

### Exclusions

You can optionally decide against generating methods, either because you want to generate less code, or because you think having something like delete / delete table available is dangerous.

```
#[derive(DynamoDb)]
#[exclusion("scan", "delete_table", "create_table")]
pub struct ExampleTestStruct {
    #[partition]
    partition_key: String,
    value: i32,
}
```

'Exclusions' accepts the following parameters: "put", "batch_put", "delete", "scan", "create_table" and "delete_table".

## Development

### Running the tests

The tests expect a local DynamoDB to be running on your machine:

```
docker run --rm -p 8000:8000 amazon/dynamodb-local
```

### PRs

Pull requests with improvements or additional features are welcome. They should at the very least add integration tests for the new functionality - and pass the existing ones!

### TODO

- batch update - or make the batch put work for both new and existing
- optional values
- better tracing of errors
- handle pagination (for query and batch)
- own error handling
    - less use of expect, more Result
    - and also tryfrom instead of from
- support boolean lists and different types of hashmaps (elegant way to do this?)
- TODO's in code

### Improvements

- allow provisioned billing in table creation
- allow decision on pub visibility of methods (default pub)
- current setup will set up a DynamoDB client for every helper struct, which is less efficient
