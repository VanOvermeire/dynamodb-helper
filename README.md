# DynamoDB Helper

## Intro

This is the outer part of the DynamoDB helper. It primarily contains testing of the procedural macro, which you can find in the `dynamodb-helper` subdirectory.

More information about usage of the macro [can be found in the readme of that directory](./dynamodb-helper/README.md)

## Status

![Github Actions Status](https://github.com/VanOvermeire/dynamodb-helper/actions/workflows/github-deploy.yml/badge.svg)

## Development

### Running the tests

The tests expect a local DynamoDB to be running on your machine:

```
docker run --rm -p 8000:8000 amazon/dynamodb-local
```

### PRs

Pull requests with improvements or additional features are welcome. They should at the very least add integration tests for the new functionality - and pass the existing ones!

### TODO

- better tracing of compile errors
- handle pagination (for query and batch)
- support more types of lists and maps
- better documentation
- TODO's in code

### Possible improvements

- more failure tests
- if an it test fails, the table is not destroyed
- when excluding, also exclude construction of errors (e.g. scan error)
- allow decision on pub visibility of methods (default pub)?
- current setup will set up a DynamoDB client for every helper struct, which is not optimal
- (low priority) support stringset, numberset, binaryset and binary