# DynamoDB Helper

This crate provides a macro that will generate a struct for interacting with DynamoDB, without all the boilerplate that that normally entails.

## Example Usage

TODO

Also see the unit and integration tests.

## TODO

- support all values
- get for complex key with only partition key
- delete call
- scan call
- own error handling
    - less use of expect, more Result
    - and also tryfrom instead of from
- help for query calls?

## Improvements

- current setup will set up a DynamoDB client for every helper struct, which is not very effective
- testing on unit level where necessary
