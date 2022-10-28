use proc_macro2::TokenStream;
use quote::quote;
use quote::__private::Ident;
use syn::punctuated::{Punctuated};
use syn::{Field};
use syn::token::Comma;
use crate::{get_relevant_field_info, possibly_optional_dynamo_type, DynamoType, IterableDynamoType};

pub fn try_from_hashmap_to_struct(struct_name: &Ident, error: &Ident, fields: &Punctuated<Field, Comma>) -> proc_macro2::TokenStream {
    let struct_inserts = fields.iter().map(|f| try_from_hashmap_for_individual_field(f, error));
    let struct_inserts_copy = struct_inserts.clone(); // quote takes ownership and we need the inserts twice...

    quote! {
        impl TryFrom<std::collections::HashMap<String, aws_sdk_dynamodb::model::AttributeValue>> for #struct_name {
            type Error = #error;

            fn try_from(map: std::collections::HashMap<String, aws_sdk_dynamodb::model::AttributeValue>) -> Result<Self, Self::Error> {
                Ok(#struct_name {
                    #(#struct_inserts)*
                })
            }
        }
        impl TryFrom<&std::collections::HashMap<String, aws_sdk_dynamodb::model::AttributeValue>> for #struct_name {
            type Error = #error;

            fn try_from(map: &std::collections::HashMap<String, aws_sdk_dynamodb::model::AttributeValue>) -> Result<Self, Self::Error> {
                Ok(#struct_name {
                    #(#struct_inserts_copy)*
                })
            }
        }
    }
}

// TODO remove the expects
fn try_from_hashmap_for_individual_field(f: &Field, err: &Ident) -> TokenStream {
    let (name, name_as_string, field_type) = get_relevant_field_info(f);

    match possibly_optional_dynamo_type(field_type) {
        crate::PossiblyOptionalDynamoType::Optional(v) => {
            match v {
                IterableDynamoType::Simple(simp) => {
                    match simp {
                        DynamoType::String => quote!(#name: map.get(#name_as_string).map(|v| v.as_s().map_err(|_| #err { message: format!("Could not convert {} from Dynamo String", #name_as_string) }).map(|v| v.to_string())).transpose()?,),
                        DynamoType::Number => quote!(#name: map.get(#name_as_string).map(|v| v.as_n().map_err(|_| #err { message: format!("Could not convert {} from Dynamo Number", #name_as_string) }).and_then(|v| str::parse(v).map_err(|_| #err { message: format!("Could not parse number for {}", #name_as_string) }))).transpose()?,),
                        DynamoType::Boolean => quote!(#name: map.get(#name_as_string).map(|v| v.as_bool().map(|v| *v).map_err(|_| #err { message: format!("Could not convert {} from Dynamo String", #name_as_string) })).transpose()?,)
                    }
                }
                // TODO these two don't work - add to compile test
                IterableDynamoType::List(simp) => {
                    build_from_hashmap_for_list_items(simp, name, name_as_string, err)
                },
                IterableDynamoType::Map(simp1, simp2) => {
                    build_from_hashmap_for_map_items(simp1, simp2, name, name_as_string, err)
                }
            }
        }
        crate::PossiblyOptionalDynamoType::Normal(v) => {
            match v {
                IterableDynamoType::Simple(simp) => {
                    match simp {
                        DynamoType::String => quote!(#name: map.get(#name_as_string).ok_or_else(|| #err { message: format!("Did not find required attribute {}", #name_as_string) })?.as_s().map_err(|_| #err { message: format!("Could not convert {} from Dynamo String", #name_as_string) }).map(|v| str::parse(v))??,),
                        DynamoType::Number => quote!(#name: map.get(#name_as_string).ok_or_else(|| #err { message: format!("Did not find required attribute {}", #name_as_string) })?.as_n().map_err(|_| #err { message: format!("Could not convert {} from Dynamo Number", #name_as_string) }).and_then(|v| str::parse(v).map_err(|_| #err { message: format!("Could not parse number for {}", #name_as_string) }))?,),
                        DynamoType::Boolean => quote!(#name: map.get(#name_as_string).ok_or_else(|| #err { message: format!("Did not find required attribute {}", #name_as_string) })?.as_bool().map(|v| *v).map_err(|_| #err { message: format!("Could not convert {} from Dynamo Boolean", #name_as_string) })?,)
                    }
                }
                IterableDynamoType::List(simp) => {
                    build_from_hashmap_for_list_items(simp, name, name_as_string, err)
                },
                IterableDynamoType::Map(simp1, simp2) => {
                    build_from_hashmap_for_map_items(simp1, simp2, name, name_as_string, err)
                }
            }
        }
    }
}

fn build_from_hashmap_for_list_items(simp: DynamoType, name: &Ident, name_as_string: String, err: &Ident) -> proc_macro2::TokenStream {
    match simp {
        DynamoType::String => {
            quote! {
                #name: map.get(#name_as_string).ok_or_else(|| #err { message: format!("Did not find required attribute {}", #name_as_string) })?.as_l().map_err(|_| #err { message: format!("Could not convert {} from Dynamo List", #name_as_string) })?.iter().map(|v| v.as_s().map_err(|_| #err { message: format!("Could not convert list element from DynamoDB string for {}", #name_as_string) }).map(|v| v.clone())).collect::<Result<Vec<_>, _>>()?,
            }
        }
        DynamoType::Number => {
            quote! {
                #name: map.get(#name_as_string).ok_or_else(|| #err { message: format!("Did not find required attribute {}", #name_as_string) })?.as_l().map_err(|_| #err { message: format!("Could not convert {} from Dynamo List", #name_as_string) })?.iter().map(|v| v.as_s().map_err(|_| #err { message: format!("Could not convert list element from DynamoDB string for {}", #name_as_string) }).and_then(|v| str::parse(v).map_err(|_| #err { message: format!("Could not convert string to number fo {}", #name_as_string) }))).collect::<Result<Vec<_>, _>>()?,
            }
        }
        _ => todo!("Only lists with strings or numbers are currently supported")
    }
}

fn build_from_hashmap_for_map_items(simp1: DynamoType, simp2: DynamoType, name: &Ident, name_as_string: String, err: &Ident) -> proc_macro2::TokenStream {
    match (simp1, simp2) {
        (DynamoType::String, DynamoType::String) => {
            quote! {
                #name: map.get(#name_as_string).ok_or_else(|| #err { message: format!("Did not find required attribute {}", #name_as_string) })?.as_m().map_err(|_| #err { message: format!("Could not convert {} from Dynamo Map", #name_as_string) })?.iter().map(|v| { if v.1.as_s().is_err() { Err(#err { message: format!("Could not convert from Dynamo String for {}", #name_as_string) }) } else { Ok((v.0.clone(), v.1.as_s().unwrap().clone())) } }).collect::<Result<HashMap<String, String>, _>>()?,
            }
        },
        _ => todo!("Only maps with strings are currently supported")
    }
}

pub fn from_struct_for_hashmap(struct_name: &Ident, fields: &Punctuated<Field, Comma>) -> proc_macro2::TokenStream {
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

fn map_insert_for(val: IterableDynamoType, name_as_string: String) -> proc_macro2::TokenStream {
    match val {
        IterableDynamoType::Simple(simp) => {
            match simp {
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
            }
        }
        IterableDynamoType::List(simp) => {
            match simp {
                DynamoType::String => {
                    quote! {
                        map.insert(#name_as_string.to_string(), aws_sdk_dynamodb::model::AttributeValue::L(to_insert.into_iter().map(|v| AttributeValue::S(v)).collect()));
                    }
                }
                DynamoType::Number => {
                    quote! {
                        map.insert(#name_as_string.to_string(), aws_sdk_dynamodb::model::AttributeValue::L(to_insert.into_iter().map(|v| AttributeValue::N(v.to_string())).collect()));
                    }
                }
                _ => todo!("Only lists with strings or numbers are currently supported")
            }
        }
        IterableDynamoType::Map(simp1, simp2) => {
            match (simp1, simp2) {
                (DynamoType::String, DynamoType::String) => {
                    quote! {
                        map.insert(#name_as_string.to_string(), aws_sdk_dynamodb::model::AttributeValue::M(to_insert.into_iter().map(|v| (v.0, AttributeValue::S(v.1))).collect()));
                    }
                },
                _ => todo!("Only maps with strings are currently supported")
            }
        }
    }
}
