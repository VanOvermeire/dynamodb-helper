use quote::quote;
use quote::__private::Ident;
use syn::{Type};
use crate::{dynamo_type, scalar_dynamo_type, DynamoTypes, DynamoScalarType};

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
        pub async fn build(region: aws_sdk_dynamodb::Region, table: &str) -> Self {
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
        pub async fn put(&self, input: #struct_name) -> Result<aws_sdk_dynamodb::output::PutItemOutput, aws_sdk_dynamodb::types::SdkError<aws_sdk_dynamodb::error::PutItemError>> {
            self.client.put_item()
                .table_name(self.table.to_string())
                .set_item(Some(input.into()))
                .send()
                .await
        }
    }
}

pub fn delete_method(struct_name: &Ident, partition_key_ident_and_type: (&Ident, &Type), range_key_ident_and_type: Option<(&Ident, &Type)>) -> proc_macro2::TokenStream {
    let partition_key_name = partition_key_ident_and_type.0.to_string();
    let partition_key_type = partition_key_ident_and_type.1;
    let partition_key_attribute_value = get_attribute_type(partition_key_type, Ident::new("partition", struct_name.span()));

    if let Some(range) = range_key_ident_and_type {
        let range_key_name = range.0.to_string();
        let range_key_type = range.1;
        let range_key_attribute_value = get_attribute_type(range_key_type, Ident::new("range", struct_name.span()));

        quote! {
            pub async fn delete(&self, partition: #partition_key_type, range: #range_key_type) -> Result<aws_sdk_dynamodb::output::DeleteItemOutput, aws_sdk_dynamodb::types::SdkError<aws_sdk_dynamodb::error::DeleteItemError>> {
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
            pub async fn delete(&self, partition: #partition_key_type) -> Result<aws_sdk_dynamodb::output::DeleteItemOutput, aws_sdk_dynamodb::types::SdkError<aws_sdk_dynamodb::error::DeleteItemError>> {
                self.client.delete_item()
                    .table_name(&self.table)
                    .key(#partition_key_name, #partition_key_attribute_value)
                    .send()
                    .await
            }
        }
    }
}

pub fn get_methods(struct_name: &Ident, partition_key_ident_and_type: (&Ident, &Type), range_key_ident_and_type: Option<(&Ident, &Type)>) -> proc_macro2::TokenStream {
    let partition_key_name = partition_key_ident_and_type.0.to_string();
    let partition_key_type = partition_key_ident_and_type.1;
    let partition_key_attribute_value = get_attribute_type(partition_key_type, Ident::new("partition", struct_name.span()));

    if let Some(range) = range_key_ident_and_type {
        let range_key_name = range.0.to_string();
        let range_key_type = range.1;
        let range_key_attribute_value = get_attribute_type(range_key_type, Ident::new("range", struct_name.span()));

        quote! {
            pub async fn get_by_partition_key(&self, partition: #partition_key_type) -> Result<Vec<#struct_name>, aws_sdk_dynamodb::types::SdkError<aws_sdk_dynamodb::error::QueryError>> {
                let result = self.client.query()
                    .table_name(&self.table)
                    .key_condition_expression("#pk = :pkval")
                    .expression_attribute_names("#pk", #partition_key_name)
                    .expression_attribute_values(":pkval", #partition_key_attribute_value)
                    .send()
                    .await?;

                let mapped_result: Vec<#struct_name> = result.items()
                    .map(|v| v.to_vec())
                    .unwrap_or_else(|| vec![])
                    .iter()
                    .map(|v| v.into())
                    .collect();

                Ok(mapped_result)
            }

            pub async fn get(&self, partition: #partition_key_type, range: #range_key_type) -> Result<Option<#struct_name>, aws_sdk_dynamodb::types::SdkError<aws_sdk_dynamodb::error::GetItemError>> {
                let result = self.client.get_item()
                    .table_name(&self.table)
                    .key(#partition_key_name, #partition_key_attribute_value)
                    .key(#range_key_name, #range_key_attribute_value)
                    .send()
                    .await?;
                Ok(result.item.map(|v| v.into()))
            }
        }
    } else {
        quote! {
            pub async fn get(&self, partition: #partition_key_type) -> Result<Option<#struct_name>, aws_sdk_dynamodb::types::SdkError<aws_sdk_dynamodb::error::GetItemError>> {
                let result = self.client.get_item()
                    .table_name(&self.table)
                    .key(#partition_key_name, #partition_key_attribute_value)
                    .send()
                    .await?;
                Ok(result.item.map(|v| v.into()))
            }
        }
    }
}

pub fn scan_method(struct_name: &Ident) -> proc_macro2::TokenStream {
    quote! {
        pub async fn scan(&self) -> Result<Vec<#struct_name>, aws_sdk_dynamodb::error::ScanError> {
            let items: Result<Vec<std::collections::HashMap<std::string::String, aws_sdk_dynamodb::model::AttributeValue>>, _> = self.client.scan()
                .table_name(&self.table)
                .into_paginator()
                .items()
                .send()
                .collect()
                .await;

            let mapped_items: Vec<#struct_name> = items.expect("TODO map tokio error onto own error").iter().map(|i| i.into()).collect(); // TODO, the error returned is prob from tokio and cannot just be mapped onto scan

            Ok(mapped_items)
        }
    }
}

pub fn create_table_method(partition_key_ident_and_type: (&Ident, &Type), range_key_ident_and_type: Option<(&Ident, &Type)>) -> proc_macro2::TokenStream {
    let partition_key_name = partition_key_ident_and_type.0.to_string();
    let partition_key_type = partition_key_ident_and_type.1;
    let partition_key_attribute_value = get_scalar_attribute(partition_key_type);

    let ads_def = if let Some(range) = range_key_ident_and_type {
        let range_key_name = range.0.to_string();
        let range_key_type = range.1;
        let range_key_attribute_value = get_scalar_attribute(range_key_type);

        quote! {
            let ads = vec![
                aws_sdk_dynamodb::model::AttributeDefinition::builder()
                    .attribute_name(#partition_key_name)
                    .attribute_type(#partition_key_attribute_value)
                    .build(),
                aws_sdk_dynamodb::model::AttributeDefinition::builder()
                    .attribute_name(#range_key_name)
                    .attribute_type(#range_key_attribute_value)
                    .build(),
            ];
        }
    } else {
        quote! {
            let ads = vec![
                aws_sdk_dynamodb::model::AttributeDefinition::builder()
                    .attribute_name(#partition_key_name)
                    .attribute_type(#partition_key_attribute_value)
                    .build(),
            ];
        }
    };

    let keys_def = if let Some(range) = range_key_ident_and_type {
        let range_key_name = range.0.to_string();

        quote! {
            let keys = vec![
                aws_sdk_dynamodb::model::KeySchemaElement::builder()
                    .key_type(aws_sdk_dynamodb::model::KeyType::Hash)
                    .attribute_name(#partition_key_name)
                    .build(),
                aws_sdk_dynamodb::model::KeySchemaElement::builder()
                    .key_type(aws_sdk_dynamodb::model::KeyType::Range)
                    .attribute_name(#range_key_name)
                    .build(),
            ];
        }
    } else {
        quote! {
            let keys = vec![
                aws_sdk_dynamodb::model::KeySchemaElement::builder()
                    .key_type(aws_sdk_dynamodb::model::KeyType::Hash)
                    .attribute_name(#partition_key_name)
                    .build()
            ];
        }
    };

    quote! {
        pub async fn create_table(&self) -> Result<aws_sdk_dynamodb::output::CreateTableOutput, aws_sdk_dynamodb::types::SdkError<aws_sdk_dynamodb::error::CreateTableError>> {
            #ads_def
            #keys_def

            self.client.create_table()
                .table_name(&self.table)
                .set_key_schema(Some(keys))
                .set_attribute_definitions(Some(ads))
                .billing_mode(aws_sdk_dynamodb::model::BillingMode::PayPerRequest)
                .send()
                .await
        }
    }
}

pub fn delete_table_method() -> proc_macro2::TokenStream {
    quote! {
        pub async fn delete_table(&self) -> Result<aws_sdk_dynamodb::output::DeleteTableOutput, aws_sdk_dynamodb::types::SdkError<aws_sdk_dynamodb::error::DeleteTableError>> {
            self.client.delete_table()
                .table_name(&self.table)
                .send()
                .await
        }
    }
}

fn get_attribute_type(key_type: &Type, name_of_attribute: Ident) -> proc_macro2::TokenStream {
    match dynamo_type(key_type) {
        DynamoTypes::String => {
            quote! {
                AttributeValue::S(#name_of_attribute)
            }
        },
        DynamoTypes::Number => {
            quote! {
                AttributeValue::N(#name_of_attribute.to_string())
            }
        },
        _ => unimplemented!("Unimplemented type")
    }
}

fn get_scalar_attribute(key_type: &Type) -> proc_macro2::TokenStream {
    match scalar_dynamo_type(key_type) {
        DynamoScalarType::String => {
            quote! {
                aws_sdk_dynamodb::model::ScalarAttributeType::S
            }
        },
        DynamoScalarType::Number => {
            quote! {
                aws_sdk_dynamodb::model::ScalarAttributeType::N
            }
        },
        DynamoScalarType::Boolean => {
            quote! {
                aws_sdk_dynamodb::model::ScalarAttributeType::B
            }
        }
    }
}
