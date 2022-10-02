# DynamoDB Helper

This crate provides a macro that will generate a struct for interacting with DynamoDB, without all the boilerplate that this normally entails.

## Example Usage

*still TODO, example from main maybe*

Also see the unit and integration tests.

Note that you will need an AWS account, credentials and these dependencies (or higher):

```
aws-config = "0.47.0"
aws-sdk-dynamodb = "0.17.0"
```

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
- `get()`
- `get_by_partition_key()` when you have a complex key (partition plus range)
- `scan()`
- `put()`

## TODO

- support all values
- delete call
- batch write
- batch read
- create and delete table call
- help for query calls?
- handle pagination
- own error handling
    - less use of expect, more Result
    - and also tryfrom instead of from

## Improvements

- allow to disable generation of methods
- current setup will set up a DynamoDB client for every helper struct, which is not very effective
- testing on unit level where necessary
- automated setup with Github Actions

## PRs

Pull requests with improvements or additional features are welcome. They should at the very least add integration tests for the new functionality - and pass the existing ones!
