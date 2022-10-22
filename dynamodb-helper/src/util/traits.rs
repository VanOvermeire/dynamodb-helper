use quote::quote;
use quote::__private::Ident;
use syn::punctuated::{Punctuated};
use syn::{Field};
use syn::token::Comma;
use crate::{get_relevant_field_info, dynamo_type, DynamoType};

// TODO try from so we can avoid the expects
pub fn build_from_hashmap_for_struct(struct_name: &Ident, fields: &Punctuated<Field, Comma>) -> proc_macro2::TokenStream {
    let struct_inserts = fields.iter().map(|f| {
        let (name, name_as_string, field_type) = get_relevant_field_info(f);

        match dynamo_type(field_type) {
            DynamoType::String => {
                quote! {
                    #name: map.get(#name_as_string).map(|v| v.as_s().expect("Attribute value conversion to work")).map(|v| v.to_string()).expect("Value for struct property to be present"),
                }
            }
            DynamoType::Number => {
                quote! {
                    #name: map.get(#name_as_string).map(|v| v.as_n().expect("Attribute value conversion to work")).map(|v| str::parse(v).expect("To be able to parse a number from Dynamo")).expect("Value for struct property to be present"),
                }
            }
            DynamoType::Boolean => {
                quote! {
                    #name: map.get(#name_as_string).map(|v| *v.as_bool().expect("Attribute value conversion to work")).expect("Value for struct property to be present"),
                }
            }
            DynamoType::StringList => {
                quote! {
                    #name: map.get(#name_as_string).map(|v| v.as_l().expect("Attribute value conversion to work")).expect("Value for struct property to be present").iter().map(|v| v.as_s().expect("Attribute value conversion to work")).map(|v| v.clone()).collect(),
                }
            }
            DynamoType::NumberList => {
                quote! {
                    #name: map.get(#name_as_string).map(|v| v.as_l().expect("Attribute value conversion to work")).expect("Value for struct property to be present").iter().map(|v| v.as_n().expect("Attribute value conversion to work")).map(|v| str::parse(v).expect("To be able to parse a number from Dynamo")).collect(),
                }
            }
            DynamoType::Map => {
                quote! {
                    #name: map.get(#name_as_string).map(|v| v.as_m().expect("Attribute value conversion to work")).expect("Value for struct property to be present").iter().map(|v| (v.0.clone(), v.1.as_s().expect("Attribute value conversion to work").clone())).collect(),
                }
            }
            _ => unimplemented!("Unimplemented type")
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

pub fn build_from_struct_for_hashmap(struct_name: &Ident, fields: &Punctuated<Field, Comma>) -> proc_macro2::TokenStream {
    let hashmap_inserts = fields.iter().map(|f| {
        let (name, name_as_string, field_type) = get_relevant_field_info(f);

        // TODO is the tostring required?
        match dynamo_type(field_type) {
            DynamoType::String => {
                quote! {
                    map.insert(#name_as_string.to_string(), aws_sdk_dynamodb::model::AttributeValue::S(input.#name));
                }
            }
            DynamoType::Number => {
                quote! {
                    map.insert(#name_as_string.to_string(), aws_sdk_dynamodb::model::AttributeValue::N(input.#name.to_string()));
                }
            }
            DynamoType::Boolean => {
                quote! {
                    map.insert(#name_as_string.to_string(), aws_sdk_dynamodb::model::AttributeValue::Bool(input.#name));
                }
            }
            DynamoType::StringList => {
                quote! {
                    map.insert(#name_as_string.to_string(), aws_sdk_dynamodb::model::AttributeValue::L(input.#name.into_iter().map(|v| AttributeValue::S(v)).collect()));
                }
            }
            DynamoType::NumberList => {
                quote! {
                    map.insert(#name_as_string.to_string(), aws_sdk_dynamodb::model::AttributeValue::L(input.#name.into_iter().map(|v| AttributeValue::N(v.to_string())).collect()));
                }
            }
            DynamoType::Map => {
                quote! {
                    map.insert(#name_as_string.to_string(), aws_sdk_dynamodb::model::AttributeValue::M(input.#name.into_iter().map(|v| (v.0, AttributeValue::S(v.1))).collect()));
                }
            }
            _ => unimplemented!("Unimplemented type")
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