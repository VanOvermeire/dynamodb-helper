use crate::{dynamo_type, scalar_dynamo_type, DynamoScalarType, DynamoType};
use proc_macro2::Ident;
use quote::quote;
use syn::Type;

pub fn new_method(helper_ident: &Ident) -> proc_macro2::TokenStream {
    quote! {
        pub fn new(client: aws_sdk_dynamodb::Client, table: &str) -> Self {
            #helper_ident {
                client,
                table: table.to_string()
            }
        }
    }
}

pub fn build_method(helper_ident: &Ident) -> proc_macro2::TokenStream {
    quote! {
        pub async fn build(region: aws_sdk_dynamodb::config::Region, table: &str) -> Self {
            let region_provider = aws_config::meta::region::RegionProviderChain::first_try(region).or_default_provider();
            let shared_config = aws_config::from_env().region(region_provider).load().await;
            #helper_ident {
                client: aws_sdk_dynamodb::Client::new(&shared_config),
                table: table.to_string(),
            }
        }
    }
}

pub fn put_method(struct_name: &Ident) -> proc_macro2::TokenStream {
    quote! {
        pub async fn put(&self, input: #struct_name) -> Result<aws_sdk_dynamodb::operation::put_item::PutItemOutput, aws_sdk_dynamodb::error::SdkError<aws_sdk_dynamodb::operation::put_item::PutItemError>> {
            self.client.put_item()
                .table_name(self.table.to_string())
                .set_item(Some(input.into()))
                .send()
                .await
        }
    }
}

pub fn delete_method(
    struct_name: &Ident,
    partition_key_ident_and_type: (&Ident, &Type),
    range_key_ident_and_type: Option<(&Ident, &Type)>,
) -> proc_macro2::TokenStream {
    let partition_key_name = partition_key_ident_and_type.0.to_string();
    let partition_key_type = partition_key_ident_and_type.1;
    let partition_key_attribute_value = get_attribute_type_for_key(partition_key_type, Ident::new("partition", struct_name.span()));

    if let Some(range) = range_key_ident_and_type {
        let range_key_name = range.0.to_string();
        let range_key_type = range.1;
        let range_key_attribute_value = get_attribute_type_for_key(range_key_type, Ident::new("range", struct_name.span()));

        quote! {
            pub async fn delete(&self, partition: #partition_key_type, range: #range_key_type) -> Result<aws_sdk_dynamodb::operation::delete_item::DeleteItemOutput, aws_sdk_dynamodb::error::SdkError<aws_sdk_dynamodb::operation::delete_item::DeleteItemError>> {
                self.client.delete_item()
                    .table_name(&self.table)
                    .key(#partition_key_name, #partition_key_attribute_value)
                    .key(#range_key_name, #range_key_attribute_value)
                    .send()
                    .await
            }
        }
    } else {
        quote! {
            pub async fn delete(&self, partition: #partition_key_type) -> Result<aws_sdk_dynamodb::operation::delete_item::DeleteItemOutput, aws_sdk_dynamodb::error::SdkError<aws_sdk_dynamodb::operation::delete_item::DeleteItemError>> {
                self.client.delete_item()
                    .table_name(&self.table)
                    .key(#partition_key_name, #partition_key_attribute_value)
                    .send()
                    .await
            }
        }
    }
}

pub fn get_methods(
    struct_name: &Ident,
    get_error: &Ident,
    get_by_partition_error: &Ident,
    partition_key_ident_and_type: (&Ident, &Type),
    range_key_ident_and_type: Option<(&Ident, &Type)>,
) -> proc_macro2::TokenStream {
    let partition_key_name = partition_key_ident_and_type.0.to_string();
    let partition_key_type = partition_key_ident_and_type.1;
    let partition_key_attribute_value = get_attribute_type_for_key(partition_key_type, Ident::new("partition", struct_name.span()));

    if let Some(range) = range_key_ident_and_type {
        let range_key_name = range.0.to_string();
        let range_key_type = range.1;
        let range_key_attribute_value = get_attribute_type_for_key(range_key_type, Ident::new("range", struct_name.span()));

        quote! {
            pub async fn get_by_partition_key(&self, partition: #partition_key_type) -> Result<Vec<#struct_name>, #get_by_partition_error> {
                let result = self.client.query()
                    .table_name(&self.table)
                    .key_condition_expression("#pk = :pkval")
                    .expression_attribute_names("#pk", #partition_key_name)
                    .expression_attribute_values(":pkval", #partition_key_attribute_value)
                    .send()
                    .await?;

                let mapped_result: Vec<#struct_name> = result
                    .items()
                    .to_vec()
                    .iter()
                    .map(|v| v.try_into())
                    .collect::<Result<Vec<_>,_>>()?;

                Ok(mapped_result)
            }

            pub async fn get(&self, partition: #partition_key_type, range: #range_key_type) -> Result<Option<#struct_name>, #get_error> {
                let result = self.client.get_item()
                    .table_name(&self.table)
                    .key(#partition_key_name, #partition_key_attribute_value)
                    .key(#range_key_name, #range_key_attribute_value)
                    .send()
                    .await?;
                let mapped = result.item.map(|v| v.try_into()).transpose()?;
                Ok(mapped)
            }
        }
    } else {
        quote! {
            pub async fn get(&self, partition: #partition_key_type) -> Result<Option<#struct_name>, #get_error> {
                let result = self.client.get_item()
                    .table_name(&self.table)
                    .key(#partition_key_name, #partition_key_attribute_value)
                    .send()
                    .await?;
                let mapped = result.item.map(|v| v.try_into()).transpose()?;
                Ok(mapped)
            }
        }
    }
}

pub fn batch_get(
    struct_name: &Ident,
    error: &Ident,
    partition_key_ident_and_type: (&Ident, &Type),
    range_key_ident_and_type: Option<(&Ident, &Type)>,
) -> proc_macro2::TokenStream {
    let partition_key_name = partition_key_ident_and_type.0.to_string();
    let partition_key_type = partition_key_ident_and_type.1;
    let partition_key_attribute_value = get_attribute_type_for_key(partition_key_type, Ident::new("partition", struct_name.span()));

    if let Some(range) = range_key_ident_and_type {
        let range_key_name = range.0.to_string();
        let range_key_type = range.1;
        let range_key_attribute_value = get_attribute_type_for_key(range_key_type, Ident::new("range", struct_name.span()));

        quote! {
            pub async fn batch_get(&self, keys: Vec<(#partition_key_type, #range_key_type)>) -> Result<Vec<#struct_name>, #error> {
                let mapped_keys: Vec<std::collections::HashMap<String, aws_sdk_dynamodb::types::AttributeValue>> = keys.into_iter().map(|(partition, range)| {
                    std::collections::HashMap::from([
                        (#partition_key_name.to_string(), #partition_key_attribute_value),
                        (#range_key_name.to_string(), #range_key_attribute_value),
                    ])
                }).collect();

                let attrs = aws_sdk_dynamodb::types::KeysAndAttributes::builder()
                    .set_keys(Some(mapped_keys))
                    .build()
                    .expect("building keys and attributes to succeed");

                let mut table_map = std::collections::HashMap::from([
                    (self.table.to_string(), attrs)
                ]);

                let result = &self.client.batch_get_item()
                    .set_request_items(Some(table_map))
                    .send()
                    .await?;

                let mapped_result: Result<Vec<_>, _> = result.responses.as_ref().and_then(|v| v.get(self.table.as_str()))
                    .map(|v| v.iter()
                        .map(|v| v.try_into())
                        .collect())
                    .unwrap_or_else(|| Ok(vec![]));

                let final_result = mapped_result?;

                Ok(final_result)
            }
        }
    } else {
        quote! {
            pub async fn batch_get(&self, keys: Vec<#partition_key_type>) -> Result<Vec<#struct_name>, #error> {
                let mapped_keys: Vec<std::collections::HashMap<String, aws_sdk_dynamodb::types::AttributeValue>> = keys.into_iter().map(|partition| {
                    std::collections::HashMap::from([
                        (#partition_key_name.to_string(), #partition_key_attribute_value)
                    ])
                }).collect();

                let attrs = aws_sdk_dynamodb::types::KeysAndAttributes::builder()
                    .set_keys(Some(mapped_keys))
                    .build()
                    .expect("building keys and attributes to succeed");

                let mut table_map = std::collections::HashMap::from([
                    (self.table.to_string(), attrs)
                ]);

                let result = &self.client.batch_get_item()
                    .set_request_items(Some(table_map))
                    .send()
                    .await?;

                let mapped_result: Result<Vec<_>, _> = result.responses.as_ref().and_then(|v| v.get(self.table.as_str()))
                    .map(|v| v.iter()
                        .map(|v| v.try_into())
                        .collect())
                    .unwrap_or_else(|| Ok(vec![]));

                let final_result = mapped_result?;

                Ok(final_result)
            }
        }
    }
}

pub fn batch_put_method(struct_name: &Ident) -> proc_macro2::TokenStream {
    quote! {
        pub async fn batch_put(&self, items: Vec<#struct_name>) -> Result<aws_sdk_dynamodb::operation::batch_write_item::BatchWriteItemOutput, aws_sdk_dynamodb::error::SdkError<aws_sdk_dynamodb::operation::batch_write_item::BatchWriteItemError>> {
            let items_as_maps: Vec<std::collections::HashMap<String, aws_sdk_dynamodb::types::AttributeValue>> = items.into_iter()
                .map(|i| i.into())
                .collect();

            let requests: Vec<aws_sdk_dynamodb::types::WriteRequest> = items_as_maps.into_iter()
                .map(|m| {
                    aws_sdk_dynamodb::types::WriteRequest::builder()
                        .put_request(aws_sdk_dynamodb::types::PutRequest::builder()
                            .set_item(Some(m))
                            .build()
                            .expect("building put request to succeed")
                        )
                        .build()
                })
                .collect();

            let mut requests_per_table = std::collections::HashMap::new();
            requests_per_table.insert(self.table.to_string(), requests);

            self.client
                .batch_write_item()
                .set_request_items(Some(requests_per_table))
                .send()
                .await
        }
    }
}

pub fn scan_method(struct_name: &Ident, error: &Ident) -> proc_macro2::TokenStream {
    quote! {
        pub async fn scan(&self) -> Result<Vec<#struct_name>, #error> {
            let items: Result<Vec<std::collections::HashMap<std::string::String, aws_sdk_dynamodb::types::AttributeValue>>, _> = self.client.scan()
                .table_name(&self.table)
                .into_paginator()
                .items()
                .send()
                .collect()
                .await;

            let final_items = items?;
            let mapped_items = final_items.iter().map(|v| v.try_into()).collect::<Result<Vec<_>, _>>()?;

            Ok(mapped_items)
        }
    }
}

pub fn create_table_method(
    partition_key_ident_and_type: (&Ident, &Type),
    range_key_ident_and_type: Option<(&Ident, &Type)>,
) -> proc_macro2::TokenStream {
    let partition_key_name = partition_key_ident_and_type.0.to_string();
    let partition_key_type = partition_key_ident_and_type.1;
    let partition_key_attribute_value = get_scalar_attribute(partition_key_type);

    let ads_def = if let Some(range) = range_key_ident_and_type {
        let range_key_name = range.0.to_string();
        let range_key_type = range.1;
        let range_key_attribute_value = get_scalar_attribute(range_key_type);

        quote! {
            let ads = vec![
                aws_sdk_dynamodb::types::AttributeDefinition::builder()
                    .attribute_name(#partition_key_name)
                    .attribute_type(#partition_key_attribute_value)
                    .build()
                    .expect("building attribute definition to succeed"),
                aws_sdk_dynamodb::types::AttributeDefinition::builder()
                    .attribute_name(#range_key_name)
                    .attribute_type(#range_key_attribute_value)
                    .build()
                    .expect("building attribute definition to succeed"),
            ];
        }
    } else {
        quote! {
            let ads = vec![
                aws_sdk_dynamodb::types::AttributeDefinition::builder()
                    .attribute_name(#partition_key_name)
                    .attribute_type(#partition_key_attribute_value)
                    .build()
                    .expect("building attribute definition to succeed"),
            ];
        }
    };

    let keys_def = if let Some(range) = range_key_ident_and_type {
        let range_key_name = range.0.to_string();

        quote! {
            let keys = vec![
                aws_sdk_dynamodb::types::KeySchemaElement::builder()
                    .key_type(aws_sdk_dynamodb::types::KeyType::Hash)
                    .attribute_name(#partition_key_name)
                    .build()
                    .expect("building keys and attributes to succeed"),
                aws_sdk_dynamodb::types::KeySchemaElement::builder()
                    .key_type(aws_sdk_dynamodb::types::KeyType::Range)
                    .attribute_name(#range_key_name)
                    .build()
                    .expect("building keys and attributes to succeed"),
            ];
        }
    } else {
        quote! {
            let keys = vec![
                aws_sdk_dynamodb::types::KeySchemaElement::builder()
                    .key_type(aws_sdk_dynamodb::types::KeyType::Hash)
                    .attribute_name(#partition_key_name)
                    .build()
                    .expect("building key schema to succeed")
            ];
        }
    };

    quote! {
        pub async fn create_table(&self) -> Result<aws_sdk_dynamodb::operation::create_table::CreateTableOutput, aws_sdk_dynamodb::error::SdkError<aws_sdk_dynamodb::operation::create_table::CreateTableError>> {
            #ads_def
            #keys_def

            self.client.create_table()
                .table_name(&self.table)
                .set_key_schema(Some(keys))
                .set_attribute_definitions(Some(ads))
                .billing_mode(aws_sdk_dynamodb::types::BillingMode::PayPerRequest)
                .send()
                .await
        }

        pub async fn create_table_with_provisioned_throughput(&self, read_capacity: i64, write_capacity: i64) -> Result<aws_sdk_dynamodb::operation::create_table::CreateTableOutput, aws_sdk_dynamodb::error::SdkError<aws_sdk_dynamodb::operation::create_table::CreateTableError>> {
            #ads_def
            #keys_def

            let provisioned = aws_sdk_dynamodb::types::ProvisionedThroughput::builder()
                .read_capacity_units(read_capacity)
                .write_capacity_units(write_capacity)
                .build()
                .expect("building provisioned throughput to succeed");

            self.client.create_table()
                .table_name(&self.table)
                .set_key_schema(Some(keys))
                .set_attribute_definitions(Some(ads))
                .billing_mode(aws_sdk_dynamodb::types::BillingMode::Provisioned)
                .provisioned_throughput(provisioned)
                .send()
                .await
        }
    }
}

pub fn delete_table_method() -> proc_macro2::TokenStream {
    quote! {
        pub async fn delete_table(&self) -> Result<aws_sdk_dynamodb::operation::delete_table::DeleteTableOutput, aws_sdk_dynamodb::error::SdkError<aws_sdk_dynamodb::operation::delete_table::DeleteTableError>> {
            self.client.delete_table()
                .table_name(&self.table)
                .send()
                .await
        }
    }
}

fn get_attribute_type_for_key(key_type: &Type, name_of_attribute: Ident) -> proc_macro2::TokenStream {
    match dynamo_type(key_type).expect("Did not find a valid DynamoDB type") {
        DynamoType::String => {
            quote! {
                aws_sdk_dynamodb::types::AttributeValue::S(#name_of_attribute)
            }
        }
        DynamoType::Number => {
            quote! {
                aws_sdk_dynamodb::types::AttributeValue::N(#name_of_attribute.to_string())
            }
        }
        DynamoType::Boolean => {
            quote! {
                aws_sdk_dynamodb::types::AttributeValue::Bool(#name_of_attribute)
            }
        }
    }
}

fn get_scalar_attribute(key_type: &Type) -> proc_macro2::TokenStream {
    match scalar_dynamo_type(key_type) {
        DynamoScalarType::String => {
            quote! {
                aws_sdk_dynamodb::types::ScalarAttributeType::S
            }
        }
        DynamoScalarType::Number => {
            quote! {
                aws_sdk_dynamodb::types::ScalarAttributeType::N
            }
        }
        DynamoScalarType::Boolean => {
            quote! {
                aws_sdk_dynamodb::types::ScalarAttributeType::B
            }
        }
    }
}
