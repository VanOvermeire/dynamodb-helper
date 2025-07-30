use crate::{get_relevant_field_info, IterableDynamoType};
use proc_macro2::Ident;
use proc_macro2::TokenStream;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{Error, Field};
use crate::implementation::dynamo_types::DynamoType;
use crate::implementation::PossiblyOptionalDynamoType;

pub fn try_from_hashmap_to_struct(struct_name: &Ident, error: &Ident, fields: &Punctuated<Field, Comma>) -> TokenStream {
    let struct_inserts = fields.iter().map(|f| try_from_hashmap_for_individual_field(f, error));
    let struct_inserts_copy = struct_inserts.clone(); // quote takes ownership and we need the inserts twice...

    quote! {
        impl TryFrom<std::collections::HashMap<String, aws_sdk_dynamodb::types::AttributeValue>> for #struct_name {
            type Error = #error;

            fn try_from(map: std::collections::HashMap<String, aws_sdk_dynamodb::types::AttributeValue>) -> Result<Self, Self::Error> {
                Ok(#struct_name {
                    #(#struct_inserts)*
                })
            }
        }
        impl TryFrom<&std::collections::HashMap<String, aws_sdk_dynamodb::types::AttributeValue>> for #struct_name {
            type Error = #error;

            fn try_from(map: &std::collections::HashMap<String, aws_sdk_dynamodb::types::AttributeValue>) -> Result<Self, Self::Error> {
                Ok(#struct_name {
                    #(#struct_inserts_copy)*
                })
            }
        }
    }
}

fn try_from_hashmap_for_individual_field(f: &Field, err: &Ident) -> TokenStream {
    let (name, name_as_string, field_type) = get_relevant_field_info(f);

    let possibly_optional_dynamo_type = match PossiblyOptionalDynamoType::try_from(field_type) {
        Ok(v) => v,
        Err(e) => {
            return e.into_compile_error().into();
        }
    };

    match possibly_optional_dynamo_type {
        PossiblyOptionalDynamoType::Optional(v) => match v {
            IterableDynamoType::Simple(simp) => match simp {
                DynamoType::String => {
                    quote!(#name: map.get(#name_as_string).map(|v| v.as_s().map_err(|_| #err::new(format!("Could not convert {} from Dynamo String", #name_as_string))).map(|v| v.to_string())).transpose()?,)
                }
                DynamoType::Number => {
                    quote!(#name: map.get(#name_as_string).map(|v| v.as_n().map_err(|_| #err::new(format!("Could not convert {} from Dynamo Number", #name_as_string))).and_then(|v| str::parse(v).map_err(|_| #err::new(format!("Could not parse number for {}", #name_as_string))))).transpose()?,)
                }
                DynamoType::Boolean => {
                    quote!(#name: map.get(#name_as_string).map(|v| v.as_bool().map(|v| *v).map_err(|_| #err::new(message: format!("Could not convert {} from Dynamo String", #name_as_string)))).transpose()?,)
                }
            },
            IterableDynamoType::List(simp) => {
                let mapping = match build_from_hashmap_for_list_items(simp, &name_as_string, err) {
                    Ok(v) => v,
                    Err(message) => Error::new(name.span(), message).into_compile_error().into(),
                };

                quote! {
                    #name: if map.get(#name_as_string).is_some() { Some(#mapping) } else { None },
                }
            }
            IterableDynamoType::Map(simp1, simp2) => {
                let mapping = match build_from_hashmap_for_map_items(simp1, simp2, &name_as_string, err) {
                    Ok(v) => v,
                    Err(message) => Error::new(name.span(), message).into_compile_error().into(),
                };

                quote! {
                    #name: if map.get(#name_as_string).is_some() { Some(#mapping) } else { None },
                }
            }
        },
        PossiblyOptionalDynamoType::Normal(v) => match v {
            IterableDynamoType::Simple(simp) => match simp {
                DynamoType::String => {
                    quote!(#name: map.get(#name_as_string).ok_or_else(|| #err::new(format!("Did not find required attribute {}", #name_as_string)))?.as_s().map_err(|_| #err::new(format!("Could not convert {} from Dynamo String", #name_as_string))).map(|v| str::parse(v))??,)
                }
                DynamoType::Number => {
                    quote!(#name: map.get(#name_as_string).ok_or_else(|| #err::new(format!("Did not find required attribute {}", #name_as_string)))?.as_n().map_err(|_| #err::new(format!("Could not convert {} from Dynamo Number", #name_as_string))).and_then(|v| str::parse(v).map_err(|_| #err::new(format!("Could not parse number for {}", #name_as_string))))?,)
                }
                DynamoType::Boolean => {
                    quote!(#name: map.get(#name_as_string).ok_or_else(|| #err::new(format!("Did not find required attribute {}", #name_as_string)))?.as_bool().map(|v| *v).map_err(|_| #err::new(format!("Could not convert {} from Dynamo Boolean", #name_as_string)))?,)
                }
            },
            IterableDynamoType::List(simp) => {
                let mapping = match build_from_hashmap_for_list_items(simp, &name_as_string, err) {
                    Ok(v) => v,
                    Err(message) => Error::new(name.span(), message).into_compile_error().into(),
                };

                quote! {
                    #name: #mapping,
                }
            }
            IterableDynamoType::Map(simp1, simp2) => {
                let mapping = match build_from_hashmap_for_map_items(simp1, simp2, &name_as_string, err) {
                    Ok(v) => v,
                    Err(message) => Error::new(name.span(), message).into_compile_error().into(),
                };

                quote! {
                    #name: #mapping,
                }
            }
        },
    }
}

fn build_from_hashmap_for_list_items(simp: DynamoType, name_as_string: &str, err: &Ident) -> Result<TokenStream, String> {
    match simp {
        DynamoType::String => {
            Ok(quote! {
                map.get(#name_as_string).ok_or_else(|| #err::new(format!("Did not find required attribute {}", #name_as_string)))?.as_l().map_err(|_| #err::new(format!("Could not convert {} from Dynamo List", #name_as_string)))?.iter().map(|v| v.as_s().map_err(|_| #err::new(format!("Could not convert list element from DynamoDB string for '{}'", #name_as_string))).map(|v| v.clone())).collect::<Result<Vec<_>, _>>()?
            })
        }
        DynamoType::Number => {
            Ok(quote! {
                map.get(#name_as_string).ok_or_else(|| #err::new(format!("Did not find required attribute {}", #name_as_string)))?.as_l().map_err(|_| #err::new(format!("Could not convert {} from Dynamo List", #name_as_string)))?.iter().map(|v| v.as_n().map_err(|_| #err::new(format!("Could not convert list element from DynamoDB string for '{}'", #name_as_string))).and_then(|v| str::parse(v).map_err(|_| #err::new(format!("Could not convert string to number fo {}", #name_as_string))))).collect::<Result<Vec<_>, _>>()?
            })
        }
        _ => Err("Only lists with strings or numbers are supported".to_string()),
    }
}

fn build_from_hashmap_for_map_items(simp1: DynamoType, simp2: DynamoType, name_as_string: &str, err: &Ident) -> Result<TokenStream, String> {
    match (simp1, simp2) {
        (DynamoType::String, DynamoType::String) => {
            Ok(quote! {
                map.get(#name_as_string).ok_or_else(|| #err::new(format!("Did not find required attribute {}", #name_as_string)))?.as_m().map_err(|_| #err::new(format!("Could not convert {} from Dynamo Map", #name_as_string)))?.iter().map(|v| { if v.1.as_s().is_err() { Err(#err::new(format!("Could not convert from Dynamo String for {}", #name_as_string))) } else { Ok((v.0.clone(), v.1.as_s().unwrap().clone())) } }).collect::<Result<HashMap<String, String>, _>>()?
            })
        }
        _ => Err("Only maps with strings are supported".to_string()),
    }
}

pub fn from_struct_for_hashmap(struct_name: &Ident, fields: &Punctuated<Field, Comma>) -> TokenStream {
    let hashmap_inserts = fields.iter().map(|f| {
        let (name, name_as_string, field_type) = get_relevant_field_info(f);

        let possibly_optional_dynamo_type = match PossiblyOptionalDynamoType::try_from(field_type) {
            Ok(v) => v,
            Err(e) => {
                return e.into_compile_error().into();
            }
        };

        match possibly_optional_dynamo_type {
            PossiblyOptionalDynamoType::Optional(v) => {
                let map_insert = match map_insert_for(v, name_as_string) {
                    Ok(v) => v,
                    Err(message) => return Error::new(name.span(), message).into_compile_error().into(),
                };
                quote! {
                    if input.#name.is_some() {
                        let to_insert = input.#name.unwrap();
                        #map_insert
                    }
                }
            }
            PossiblyOptionalDynamoType::Normal(v) => {
                let map_insert = match map_insert_for(v, name_as_string) {
                    Ok(v) => v,
                    Err(message) => return Error::new(name.span(), message).into_compile_error().into(),
                };
                quote! {
                    let to_insert = input.#name;
                    #map_insert
                }
            }
        }
    });

    quote! {
        impl From<#struct_name> for std::collections::HashMap<String, aws_sdk_dynamodb::types::AttributeValue> {
            fn from(input: #struct_name) -> Self {
                let mut map = std::collections::HashMap::new();
                #(#hashmap_inserts)*
                map
            }
        }
    }
}

fn map_insert_for(val: IterableDynamoType, name_as_string: String) -> Result<TokenStream, String> {
    let result = match val {
        IterableDynamoType::Simple(simp) => match simp {
            DynamoType::String => {
                quote! {
                    map.insert(#name_as_string.to_string(), aws_sdk_dynamodb::types::AttributeValue::S(to_insert));
                }
            }
            DynamoType::Number => {
                quote! {
                    map.insert(#name_as_string.to_string(), aws_sdk_dynamodb::types::AttributeValue::N(to_insert.to_string()));
                }
            }
            DynamoType::Boolean => {
                quote! {
                    map.insert(#name_as_string.to_string(), aws_sdk_dynamodb::types::AttributeValue::Bool(to_insert));
                }
            }
        },
        IterableDynamoType::List(simp) => match simp {
            DynamoType::String => {
                quote! {
                    map.insert(#name_as_string.to_string(), aws_sdk_dynamodb::types::AttributeValue::L(to_insert.into_iter().map(|v| aws_sdk_dynamodb::types::AttributeValue::S(v)).collect()));
                }
            }
            DynamoType::Number => {
                quote! {
                    map.insert(#name_as_string.to_string(), aws_sdk_dynamodb::types::AttributeValue::L(to_insert.into_iter().map(|v| aws_sdk_dynamodb::types::AttributeValue::N(v.to_string())).collect()));
                }
            }
            _ => return Err("Only lists with strings or numbers are supported".to_string()),
        },
        IterableDynamoType::Map(simp1, simp2) => match (simp1, simp2) {
            (DynamoType::String, DynamoType::String) => {
                quote! {
                    map.insert(#name_as_string.to_string(), aws_sdk_dynamodb::types::AttributeValue::M(to_insert.into_iter().map(|v| (v.0, aws_sdk_dynamodb::types::AttributeValue::S(v.1))).collect()));
                }
            }
            _ => return Err("Only maps with strings are supported".to_string()),
        },
    };
    Ok(result)
}
