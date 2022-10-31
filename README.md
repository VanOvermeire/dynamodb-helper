# DynamoDB Helper

## Intro

This is the outer part of the DynamoDB helper. It primarily contains testing of the procedural macro, which you can find in the `dynamodb-helper` subdirectory.

More information about usage of the macro [can be found in the readme of that directory](./dynamodb-helper/README.md). 

The `main.rs` of this outer package contains an example running against the real AWS environment. A unit test and integration tests contain more examples.

## Status

![Github Actions Status](https://github.com/VanOvermeire/dynamodb-helper/actions/workflows/github-deploy.yml/badge.svg)

## Development

### Running the tests

The tests expect a local DynamoDB to be running on your machine:

```
docker run --rm -p 8000:8000 amazon/dynamodb-local
```

### PRs

Pull requests with improvements or additional features are appreciated. They should at the very least add integration tests for the new functionality - and pass the existing ones!

### TODO

- handle pagination for query and batch
- support more types of lists and maps
  - also support nested structs? newtypes!
- documentation: exact signatures of all the methods and traits
- allow to change names as they are saved in DynamoDB
- add clippy

### Possible improvements

(But not very high priority...)

- if an it test panics, the table is not destroyed...
- current setup will set up a DynamoDB client for every helper struct, which is not optimal
- allow decision on pub visibility of methods (default pub)?
- support stringset, numberset, binaryset and binary
