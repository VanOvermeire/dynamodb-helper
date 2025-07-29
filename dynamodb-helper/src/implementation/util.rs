use crate::{BATCH_GET_METHOD_NAME, BATCH_PUT_METHOD_NAME, GET_METHOD_NAME, PUT_METHOD_NAME, SCAN_METHOD_NAME};
use proc_macro2::Ident;
use proc_macro2::TokenTree::Literal;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::PathArguments::AngleBracketed;
use syn::__private::TokenStream2;
use syn::{AngleBracketedGenericArguments, Field};
use syn::{Attribute, Meta};

pub const ALL_NUMERIC_TYPES_AS_STRINGS: &[&str] = &["u8", "u16", "u32", "u64", "u128", "i8", "i16", "i32", "i64", "i128", "f32", "f64"];

#[derive(Debug)]
pub enum DynamoType {
    Number,
    String,
    Boolean,
}

#[derive(Debug)]
pub enum PossiblyOptionalDynamoType {
    Normal(IterableDynamoType),
    Optional(IterableDynamoType),
}

#[derive(Debug)]
pub enum IterableDynamoType {
    Simple(DynamoType),
    List(DynamoType),
    Map(DynamoType, DynamoType),
}

pub fn possibly_optional_dynamo_type(ty: &syn::Type) -> PossiblyOptionalDynamoType {
    if matches_type(ty, "Option") {
        if let syn::Type::Path(ref p) = ty {
            if let AngleBracketed(AngleBracketedGenericArguments { args, .. }) = &p.path.segments[0].arguments {
                return match &args[0] {
                    syn::GenericArgument::Type(t) => PossiblyOptionalDynamoType::Optional(iterable_dynamo_type(t)),
                    _ => unreachable!("Option should have an inner type - but we did not find one"),
                };
            }
        }
        unreachable!("Option should have inner type - but we did not find one");
    } else {
        PossiblyOptionalDynamoType::Normal(iterable_dynamo_type(ty))
    }
}

fn iterable_dynamo_type(ty: &syn::Type) -> IterableDynamoType {
    if let syn::Type::Path(ref p) = ty {
        let first_match = p.path.segments[0].ident.to_string();

        if first_match == "Vec" {
            if let AngleBracketed(AngleBracketedGenericArguments { args, .. }) = &p.path.segments[0].arguments {
                return match &args[0] {
                    syn::GenericArgument::Type(t) => IterableDynamoType::List(dynamo_type(t).expect("Did not find a valid DynamoDB type")),
                    _ => unreachable!("Vec should have an inner type - but we did not find one"),
                };
            }
        } else if first_match == "HashMap" {
            if let AngleBracketed(AngleBracketedGenericArguments { args, .. }) = &p.path.segments[0].arguments {
                let map_args: Vec<Option<&syn::Type>> = args
                    .iter()
                    .map(|rabbit_hole| match rabbit_hole {
                        syn::GenericArgument::Type(t) => Some(t),
                        _ => None,
                    })
                    .collect();
                return IterableDynamoType::Map(
                    dynamo_type(map_args[0].expect("We expect HashMap to have a first argument - but we did not find one")).expect("Did not find a valid DynamoDB type"),
                    dynamo_type(map_args[1].expect("We expect HashMap to have a second argument - but we did not find one")).expect("Did not find a valid DynamoDB type"),
                );
            }
        }
    }
    IterableDynamoType::Simple(dynamo_type(ty).expect("Did not find a valid DynamoDB type"))
}

pub fn dynamo_type(ty: &syn::Type) -> Option<DynamoType> {
    if matches_any_type(ty, ALL_NUMERIC_TYPES_AS_STRINGS.to_vec()) {
        Some(DynamoType::Number)
    } else if matches_type(ty, "bool") {
        Some(DynamoType::Boolean)
    } else if matches_type(ty, "String") {
        Some(DynamoType::String)
    } else {
        None
    }
}

pub fn get_ident_and_type_of_field_annotated_with<'a>(
    fields: &'a Punctuated<Field, Comma>,
    name: &'a str,
) -> Option<(&'a Ident, &'a syn::Type)> {
    fields
        .iter()
        .filter(|f| get_attribute(f, name).is_some())
        .map(|f| (f.ident.as_ref().unwrap(), &f.ty))
        .next()
}

fn get_attribute<'a>(f: &'a Field, name: &'a str) -> Option<&'a Attribute> {
    f.attrs
        .iter()
        .find(|&attr| attr.path().segments.len() == 1 && attr.path().segments[0].ident == name)
}

pub fn get_relevant_field_info(f: &Field) -> (&Ident, String, &syn::Type) {
    let name = &f.ident.as_ref().unwrap();
    let name_as_string = name.to_string();
    let field_type = &f.ty;
    (name, name_as_string, field_type)
}

pub fn matches_any_type(ty: &syn::Type, type_names: Vec<&str>) -> bool {
    type_names.iter().any(|v| matches_type(ty, v))
}

pub fn matches_type(ty: &syn::Type, type_name: &str) -> bool {
    if let syn::Type::Path(ref p) = ty {
        let first_match = p.path.segments[0].ident.to_string();
        return first_match == type_name;
    }
    false
}

pub fn get_macro_attribute(attrs: &[Attribute], attribute_name: &str) -> Vec<String> {
    attrs
        .iter()
        .filter(|attribute| attribute.path().is_ident(attribute_name))
        .flat_map(|attribute| match &attribute.meta {
            Meta::List(l) => l
                .clone()
                .tokens
                .into_iter()
                .filter_map(|s| match s {
                    Literal(l) => Some(l.to_string()),
                    _ => None,
                })
                .collect(),
            _ => vec![],
        })
        .map(|att| att.replace("\"", "")) // caused by the to_string, probably a better way to do this
        .collect()
}

pub fn tokenstream_or_empty_if_no_retrieval_methods(stream: TokenStream2, exclusions: &[&str]) -> TokenStream2 {
    tokenstream_or_empty_if_boolean_function(stream, &|| {
        exclusions.contains(&GET_METHOD_NAME) && exclusions.contains(&BATCH_GET_METHOD_NAME) && exclusions.contains(&SCAN_METHOD_NAME)
    })
}

pub fn tokenstream_or_empty_if_no_put_methods(stream: TokenStream2, exclusions: &[&str]) -> TokenStream2 {
    tokenstream_or_empty_if_boolean_function(stream, &|| {
        exclusions.contains(&PUT_METHOD_NAME) && exclusions.contains(&BATCH_PUT_METHOD_NAME)
    })
}

pub fn tokenstream_or_empty_if_exclusion(stream: TokenStream2, method_name: &str, exclusions: &[&str]) -> TokenStream2 {
    tokenstream_or_empty_if_boolean_function(stream, &|| exclusions.contains(&method_name))
}

pub fn tokenstream_or_empty_if_boolean_function(stream: TokenStream2, fun: &dyn Fn() -> bool) -> TokenStream2 {
    if fun() {
        quote! {}
    } else {
        stream
    }
}
