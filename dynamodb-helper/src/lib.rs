extern crate core;

use quote::quote;
use proc_macro::{TokenStream};
use std::collections::HashMap;
use std::iter::Map;
use quote::__private::Ident;
use syn::{parse_macro_input, DeriveInput, Field, Type};
use syn::Data::Struct;
use syn::DataStruct;
use syn::Fields::Named;
use syn::FieldsNamed;
use syn::Meta::NameValue;
use syn::MetaNameValue;
use syn::Lit;
use syn::punctuated::{Iter, Punctuated};
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
    let get = get_method(&name, partition_key_ident_and_type, range_key_ident_and_type);
    let put = put_method(&name);

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
            #put
            #get
        }
    };

    public_version.into()
}

fn new_method(helper_ident: &Ident) -> proc_macro2::TokenStream {
    quote! {
        fn new(client: aws_sdk_dynamodb::Client, table: &str) -> Self {
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

fn get_method(struct_name: &Ident, partition_key_ident_and_type: (&Ident, &Type), range_key_ident_and_type: Option<(&Ident, &Type)>) -> proc_macro2::TokenStream {
    let partition_key_name = partition_key_ident_and_type.0.to_string();
    let partition_key_type = partition_key_ident_and_type.1;
    let partition_key_attribute_value = if matches_any_type(partition_key_type, ALL_NUMERIC_TYPES_AS_STRINGS.to_vec()) {
        quote! {
            AttributeValue::N(partition.to_string())
        }
    } else {
        quote! {
            AttributeValue::S(partition)
        }
    };

    if let Some(range) = range_key_ident_and_type {
        let range_key_name = range.0.to_string();
        let range_key_type = range.1;
        let range_key_attribute_value = if matches_any_type(range_key_type, ALL_NUMERIC_TYPES_AS_STRINGS.to_vec()) {
            quote! {
                AttributeValue::N(range.to_string())
        }
        } else {
            quote! {
                AttributeValue::S(range)
            }
        };

        quote! {
            pub async fn get(&self, partition: #partition_key_type, range: #range_key_type) -> Result<#struct_name, aws_sdk_dynamodb::types::SdkError<aws_sdk_dynamodb::error::GetItemError>> {
                let result = self.client.get_item()
                    .table_name(&self.table)
                    .key(#partition_key_name, #partition_key_attribute_value)
                    .key(#range_key_name, #range_key_attribute_value)
                    .send()
                    .await?;

                let mappie = result.item.expect("Just temp"); // TODO transform into error
                Ok(mappie.into())
            }
        }
    } else {
        quote! {
            pub async fn get(&self, partition: #partition_key_type) -> Result<#struct_name, aws_sdk_dynamodb::types::SdkError<aws_sdk_dynamodb::error::GetItemError>> {
                let result = self.client.get_item()
                    .table_name(&self.table)
                    .key(#partition_key_name, #partition_key_attribute_value)
                    .send()
                    .await?;
                let mappie = result.item.expect("Just temp"); // TODO transform into error
                Ok(mappie.into())
            }
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

    quote! {
        impl From<std::collections::HashMap<String, aws_sdk_dynamodb::model::AttributeValue>> for #struct_name {
            fn from(map: std::collections::HashMap<String, aws_sdk_dynamodb::model::AttributeValue>) -> Self {
                #struct_name {
                    #(#struct_inserts)*
                }
            }
        }
    }
}

fn build_from_struct_for_hashmap(struct_name: &Ident, fields: &Punctuated<Field, Comma>) -> proc_macro2::TokenStream {
    let hashmap_inserts = fields.iter().map(|f| {
        let (name, name_as_string, field_type) = get_relevant_field_info(f);

        // TODO handle other types like booleans
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

fn get_ident_and_type_of_field_annotated_with<'a>(fields: &'a Punctuated<Field, Comma>, name: &'a str) -> Option<(&'a Ident, &'a Type)> {
    fields.iter()
        .filter(|f| get_attribute(f, name).is_some())
        .map(|f| (f.ident.as_ref().unwrap(), &f.ty))
        .next()
}

fn get_relevant_field_info<'a>(f: &'a Field) -> (&'a Ident, String, &Type) {
    let name = &f.ident.as_ref().unwrap();
    let name_as_string = name.to_string();
    let field_type = &f.ty;
    (name, name_as_string, field_type)
}

fn get_attribute<'a>(f: &'a syn::Field, name: &'a str) -> Option<&'a syn::Attribute> {
    for attr in &f.attrs {
        if attr.path.segments.len() == 1 && attr.path.segments[0].ident == name.to_string() {
            return Some(attr);
        }
    }
    None
}

fn matches_any_type<'a>(ty: &'a syn::Type, type_names: Vec<&str>) -> bool {
    type_names.iter().any(|v| matches_type(ty, v))
}

fn matches_type<'a>(ty: &'a syn::Type, type_name: &str) -> bool {
    if let syn::Type::Path(ref p) = ty {
        return p.path.segments[0].ident.to_string() == type_name.to_string()
    }
    false
}
