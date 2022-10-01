# DynamoDB Helper

This crate provides a macro that will generate a struct for interacting with DynamoDB, without all the boilerplate that that normally entails.

## Example Usage

TODO

Also see the unit and integration tests.

## TODO

- support range keys
- support values besides String
- delete call
- scan call
- more useful return from get
- help for query calls?
- less use of expect

## Improvements

Current setup will set up a DynamoDB client for every helper struct, which is not very effective.
Improve testing
