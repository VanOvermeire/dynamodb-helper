extern crate core;

mod util;

use crate::util::{get_ident_and_type_of_field_annotated_with, get_relevant_field_info, matches_any_type, matches_type};
use quote::quote;
use proc_macro::{TokenStream};
use quote::__private::Ident;
use syn::{parse_macro_input, DeriveInput, Field, Type};
use syn::Data::Struct;
use syn::DataStruct;
use syn::Fields::Named;
use syn::FieldsNamed;
use syn::punctuated::{Punctuated};
use syn::spanned::Spanned;
use syn::token::Comma;

const ALL_NUMERIC_TYPES_AS_STRINGS: &'static [&'static str] = &["u8", "u16", "u32", "u64", "u128", "i8", "i16", "i32", "i64", "i128", "f32", "f64"];

#[proc_macro_derive(DynamoDb, attributes(partition,range))]
pub fn create_dynamodb_helper(item: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(item as DeriveInput);
    let name = ast.ident;
    let helper_name = format!("{}Db", name);
    let helper_ident = syn::Ident::new(&helper_name, name.span());

    let fields = match ast.data {
        Struct(DataStruct { fields: Named(FieldsNamed { ref named, .. }), .. }) => named,
        _ => unimplemented!("Only works for structs"),
    };

    let partition_key_ident_and_type = get_ident_and_type_of_field_annotated_with(fields, "partition").expect("Partition key should be defined (with attribute #[partition])");
    let range_key_ident_and_type = get_ident_and_type_of_field_annotated_with(fields, "range");

    let from_struct_for_hashmap = build_from_struct_for_hashmap(&name, fields);
    let from_hashmap_for_struct = build_from_hashmap_for_struct(&name, fields);

    let new = new_method(&helper_ident);
    let build = build_method(&helper_ident);

    let create_table = create_table_method(partition_key_ident_and_type, range_key_ident_and_type);
    let delete_table = delete_table_method();

    let gets = get_methods(&name, partition_key_ident_and_type, range_key_ident_and_type);
    let put = put_method(&name);
    let scan = scan_method(&name);

    let public_version = quote! {
        #from_struct_for_hashmap
        #from_hashmap_for_struct

        pub struct #helper_ident {
            client: aws_sdk_dynamodb::Client,
            table: String,
        }

        impl #helper_ident {
            #new
            #build

            #create_table
            #delete_table

            #put
            #gets
            #scan
        }
    };

    public_version.into()
}

fn new_method(helper_ident: &Ident) -> proc_macro2::TokenStream {
    quote! {
        pub fn new(client: aws_sdk_dynamodb::Client, table: &str) -> Self {
            #helper_ident {
                client,
                table: table.to_string()
            }
        }
    }
}

fn build_method(helper_ident: &Ident) -> proc_macro2::TokenStream {
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

fn put_method(struct_name: &Ident) -> proc_macro2::TokenStream {
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

fn get_methods(struct_name: &Ident, partition_key_ident_and_type: (&Ident, &Type), range_key_ident_and_type: Option<(&Ident, &Type)>) -> proc_macro2::TokenStream {
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

fn scan_method(struct_name: &Ident) -> proc_macro2::TokenStream {
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

fn create_table_method(partition_key_ident_and_type: (&Ident, &Type), range_key_ident_and_type: Option<(&Ident, &Type)>) -> proc_macro2::TokenStream {
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

fn delete_table_method() -> proc_macro2::TokenStream {
    quote! {
        pub async fn delete_table(&self) -> Result<aws_sdk_dynamodb::output::DeleteTableOutput, aws_sdk_dynamodb::types::SdkError<aws_sdk_dynamodb::error::DeleteTableError>> {
            self.client.delete_table()
                .table_name(&self.table)
                .send()
                .await
        }
    }
}

fn build_from_hashmap_for_struct(struct_name: &Ident, fields: &Punctuated<Field, Comma>) -> proc_macro2::TokenStream {
    let struct_inserts = fields.iter().map(|f| {
        let (name, name_as_string, field_type) = get_relevant_field_info(f);

        if matches_any_type(field_type, ALL_NUMERIC_TYPES_AS_STRINGS.to_vec()) {
            quote! {
                #name: map.get(#name_as_string).map(|v| v.as_n().expect("Attribute value conversion to work")).map(|v| str::parse(v).expect("To be able to parse a number from Dynamo")).expect("Value for struct property to be present"),
            }
        } else {
            // default to string
            quote! {
                #name: map.get(#name_as_string).map(|v| v.as_s().expect("Attribute value conversion to work")).map(|v| v.to_string()).expect("Value for struct property to be present"),
            }
        }
    });
    let struct_inserts_copy = struct_inserts.clone(); // quote takes ownership

    quote! {
        impl From<std::collections::HashMap<String, aws_sdk_dynamodb::model::AttributeValue>> for #struct_name {
            fn from(map: std::collections::HashMap<String, aws_sdk_dynamodb::model::AttributeValue>) -> Self {
                #struct_name {
                    #(#struct_inserts)*
                }
            }
        }

        impl From<&std::collections::HashMap<String, aws_sdk_dynamodb::model::AttributeValue>> for #struct_name {
            fn from(map: &std::collections::HashMap<String, AttributeValue>) -> Self {
                #struct_name {
                    #(#struct_inserts_copy)*
                }
            }
        }
    }
}

fn build_from_struct_for_hashmap(struct_name: &Ident, fields: &Punctuated<Field, Comma>) -> proc_macro2::TokenStream {
    let hashmap_inserts = fields.iter().map(|f| {
        let (name, name_as_string, field_type) = get_relevant_field_info(f);

        // TODO handle other types like booleans
        //  maybe matches returns an enum that we can 'match' on
        if matches_any_type(field_type, ALL_NUMERIC_TYPES_AS_STRINGS.to_vec()) {
            quote! {
                map.insert(#name_as_string.to_string(), aws_sdk_dynamodb::model::AttributeValue::N(input.#name.to_string()));
            }
        } else {
            // default to string
            quote! {
                map.insert(#name_as_string.to_string(), aws_sdk_dynamodb::model::AttributeValue::S(input.#name));
            }
        }
    });

    quote! {
        impl From<#struct_name> for std::collections::HashMap<String, aws_sdk_dynamodb::model::AttributeValue> {
            fn from(input: #struct_name) -> Self {
                let mut map = std::collections::HashMap::new();
                #(#hashmap_inserts)*
                map
            }
        }
    }
}

fn get_attribute_type(key_type: &Type, name_of_attribute: Ident) -> proc_macro2::TokenStream {
    if matches_any_type(key_type, ALL_NUMERIC_TYPES_AS_STRINGS.to_vec()) {
        quote! {
            AttributeValue::N(#name_of_attribute.to_string())
        }
    } else {
        quote! {
            AttributeValue::S(#name_of_attribute)
        }
    }
}

fn get_scalar_attribute(key_type: &Type) -> proc_macro2::TokenStream {
    if matches_any_type(key_type, ALL_NUMERIC_TYPES_AS_STRINGS.to_vec()) {
        quote! {
            aws_sdk_dynamodb::model::ScalarAttributeType::N
        }
    } else if matches_type(key_type, "bool") {
        quote! {
            aws_sdk_dynamodb::model::ScalarAttributeType::B
        }
    }
    else {
        quote! {
            aws_sdk_dynamodb::model::ScalarAttributeType::S
        }
    }
}
