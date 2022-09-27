# DynamoDB Helper

This crate provides a macro that will generate a struct for interacting with DynamoDB, without all the boilerplate that that normally entails.

## Example Usage

...

## TODO

- support range keys
- support values besides String
- valid delete call
- help for query calls?

## Improvements

Current setup will set up a DynamoDB client for every helper struct, which is not very effective.
Improve testing
