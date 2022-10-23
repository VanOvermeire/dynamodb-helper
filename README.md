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

- better tracing of errors
- handle pagination (for query and batch)
- own error handling
  - less use of expect, more Result
  - and also tryfrom instead of from
- support more types of lists and maps
- TODO's in code

### Possible improvements

- allow decision on pub visibility of methods (default pub)?
- current setup will set up a DynamoDB client for every helper struct, which is not optimal
- support stringset, numberset, binaryset (low priority) and binary
