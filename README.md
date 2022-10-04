# DynamoDB Helper

This crate provides a macro that will generate a struct for interacting with DynamoDB, without all the boilerplate that this normally entails.

// compared with popular rust library for doing this (uses official sdk)

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

Note that you will need an AWS account, credentials and these dependencies:

```
aws-config = "0.47.0"
aws-sdk-dynamodb = "0.17.0"
```

Be sure to check the important notes and details below for more usage info and an overview of attributes and methods.

## Important notes

Certain methods like scan required the following trait to be in scope, so you will need to add this to your file

```
use tokio_stream::StreamExt;
```

For which you will need the `tokio-stream` dependency.

If you get an error warning about `the following trait bounds were not satisfied: impl futures_core::stream::Stream<Item = Result<...> + Unpin: Iterator`

In the future, you will be able to disable generation of these methods as an alternative.

## Details

### Attributes

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
- `scan()`
- `put()`
- `delete()`

The create and delete table methods are appropriate for testing, pocs and smaller projects. For real applications it is probably better to create the tables as IAC and to pass the names to `new()` or `build()`.

## PRs

Pull requests with improvements or additional features are welcome. They should at the very least add integration tests for the new functionality - and pass the existing ones!

## TODO

- support all values (now just numbers, strings and boolean partially)
  - allow override of type (overlap between set and list) 
- batch write
- batch read
- handle pagination (for query and batch)
- own error handling
    - less use of expect, more Result
    - and also tryfrom instead of from

## Improvements

- help for query calls?
- allow provisioned billing in table creation
- allow decision on pub visibility of methods (default pub)?
- allow to disable generation of methods
- current setup will set up a DynamoDB client for every helper struct, which is not very effective
- testing on unit level where necessary
- automated setup with Github Actions
