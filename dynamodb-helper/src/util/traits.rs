use quote::quote;
use quote::__private::Ident;
use syn::punctuated::{Punctuated};
use syn::{Field};
use syn::token::Comma;
use crate::{get_relevant_field_info, dynamo_type, DynamoTypes};

pub fn build_from_hashmap_for_struct(struct_name: &Ident, fields: &Punctuated<Field, Comma>) -> proc_macro2::TokenStream {
    let struct_inserts = fields.iter().map(|f| {
        let (name, name_as_string, field_type) = get_relevant_field_info(f);

        match dynamo_type(field_type) {
            DynamoTypes::String => {
                quote! {
                    #name: map.get(#name_as_string).map(|v| v.as_s().expect("Attribute value conversion to work")).map(|v| v.to_string()).expect("Value for struct property to be present"),
                }
            },
            DynamoTypes::Number => {
                quote! {
                    #name: map.get(#name_as_string).map(|v| v.as_n().expect("Attribute value conversion to work")).map(|v| str::parse(v).expect("To be able to parse a number from Dynamo")).expect("Value for struct property to be present"),
                }
            },
            DynamoTypes::Boolean => {
                quote! {
                    #name: map.get(#name_as_string).map(|v| *v.as_bool().expect("Attribute value conversion to work")).expect("Value for struct property to be present"),
                }
            },
            DynamoTypes::StringSet => {
                quote! {
                    #name: map.get(#name_as_string).map(|v| v.as_ss().expect("Attribute value conversion to work")).expect("Value for struct property to be present"),
                }
            },
            DynamoTypes::NumberSet => {
                quote! {
                    #name: map.get(#name_as_string).map(|v| v.as_ns().expect("Attribute value conversion to work")).expect("Value for struct property to be present").iter().map(|v| str::parse(v)).collect(),
                }
            },
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

        match dynamo_type(field_type) {
            DynamoTypes::String => {
                quote! {
                    map.insert(#name_as_string.to_string(), aws_sdk_dynamodb::model::AttributeValue::S(input.#name));
                }
            },
            DynamoTypes::Number => {
                quote! {
                    map.insert(#name_as_string.to_string(), aws_sdk_dynamodb::model::AttributeValue::N(input.#name.to_string()));
                }
            },
            DynamoTypes::Boolean => {
                quote! {
                    map.insert(#name_as_string.to_string(), aws_sdk_dynamodb::model::AttributeValue::Bool(input.#name));
                }
            },
            DynamoTypes::StringSet => {
                quote! {
                    map.insert(#name_as_string.to_string(), aws_sdk_dynamodb::model::AttributeValue::Ss(input.#name));
                }
            },
            DynamoTypes::NumberSet => {
                quote! {
                    map.insert(#name_as_string.to_string(), aws_sdk_dynamodb::model::AttributeValue::Ns(input.#name));
                }
            },
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