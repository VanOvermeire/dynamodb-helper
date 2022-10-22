use quote::quote;
use quote::__private::Ident;
use syn::punctuated::{Punctuated};
use syn::{Field};
use syn::token::Comma;
use crate::{get_relevant_field_info, dynamo_type, possibly_optional_dynamo_type, DynamoType};

// TODO try from so we can avoid the expects
// TODO maybe we can make the match more elegant if we split of the list/map stuff later
pub fn build_from_hashmap_for_struct(struct_name: &Ident, fields: &Punctuated<Field, Comma>) -> proc_macro2::TokenStream {
    let struct_inserts = fields.iter().map(|f| {
        let (name, name_as_string, field_type) = get_relevant_field_info(f);

        match possibly_optional_dynamo_type(field_type) {
            crate::PossiblyOptionalDynamoType::Optional(v) => {
                match v {
                    DynamoType::String => {
                        quote! {
                            #name: map.get(#name_as_string).map(|v| v.as_s().expect("Attribute value conversion to work")).map(|v| v.to_string()),
                        }
                    }
                    DynamoType::Number => {
                        quote! {
                            #name: map.get(#name_as_string).map(|v| v.as_n().expect("Attribute value conversion to work")).map(|v| str::parse(v).expect("To be able to parse a number from Dynamo")),
                        }
                    }
                    DynamoType::Boolean => {
                        quote! {
                            #name: map.get(#name_as_string).map(|v| *v.as_bool().expect("Attribute value conversion to work")),
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
            }
            crate::PossiblyOptionalDynamoType::Normal(v) => {
                match v {
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

// TODO are the tostring required?
pub fn build_from_struct_for_hashmap(struct_name: &Ident, fields: &Punctuated<Field, Comma>) -> proc_macro2::TokenStream {
    let hashmap_inserts = fields.iter().map(|f| {
        let (name, name_as_string, field_type) = get_relevant_field_info(f);

        match possibly_optional_dynamo_type(field_type) {
            crate::PossiblyOptionalDynamoType::Optional(v) => {
                let map_insert = map_insert_for(v, name_as_string);
                quote! {
                    if input.#name.is_some() {
                        let to_insert = input.#name.unwrap();
                        #map_insert
                    }
                }
            }
            crate::PossiblyOptionalDynamoType::Normal(v) => {
                let map_insert = map_insert_for(v, name_as_string);
                quote! {
                    let to_insert = input.#name;
                    #map_insert
                }
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

fn map_insert_for(val: DynamoType, name_as_string: String) -> proc_macro2::TokenStream {
    match val {
        DynamoType::String => {
            quote! {
                    map.insert(#name_as_string.to_string(), aws_sdk_dynamodb::model::AttributeValue::S(to_insert));
                }
        }
        DynamoType::Number => {
            quote! {
                    map.insert(#name_as_string.to_string(), aws_sdk_dynamodb::model::AttributeValue::N(to_insert.to_string()));
                }
        }
        DynamoType::Boolean => {
            quote! {
                    map.insert(#name_as_string.to_string(), aws_sdk_dynamodb::model::AttributeValue::Bool(to_insert));
                }
        }
        DynamoType::StringList => {
            quote! {
                    map.insert(#name_as_string.to_string(), aws_sdk_dynamodb::model::AttributeValue::L(to_insert.into_iter().map(|v| AttributeValue::S(v)).collect()));
                }
        }
        DynamoType::NumberList => {
            quote! {
                    map.insert(#name_as_string.to_string(), aws_sdk_dynamodb::model::AttributeValue::L(to_insert.into_iter().map(|v| AttributeValue::N(v.to_string())).collect()));
                }
        }
        DynamoType::Map => {
            quote! {
                    map.insert(#name_as_string.to_string(), aws_sdk_dynamodb::model::AttributeValue::M(to_insert.into_iter().map(|v| (v.0, AttributeValue::S(v.1))).collect()));
                }
        }
        _ => unimplemented!("Unimplemented type")
    }
}
