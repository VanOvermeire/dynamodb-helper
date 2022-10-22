use quote::quote;
use quote::__private::Ident;
use syn::{Attribute};
use syn::{AngleBracketedGenericArguments, Field};
use syn::PathArguments::AngleBracketed;
use syn::punctuated::{Punctuated};
use syn::token::Comma;
use syn::__private::TokenStream2;
use proc_macro2::TokenTree::Group;
use proc_macro2::TokenTree::Literal;
use quote::__private::ext::RepToTokensExt;

pub const ALL_NUMERIC_TYPES_AS_STRINGS: &'static [&'static str] = &["u8", "u16", "u32", "u64", "u128", "i8", "i16", "i32", "i64", "i128", "f32", "f64"];

pub enum DynamoScalarType {
    Number,
    String,
    Boolean,
}

pub enum DynamoType {
    Number,
    Boolean,
    StringList,
    NumberList,
    List,
    Map,
    String,
}

// TODO could possibly use a similar approach for hashmap and list...
// IterarableDynamoType { Simple(DynamoType), List(DynamoType), Map(DynamoType, DynamoType)
pub enum PossiblyOptionalDynamoType {
    Normal(DynamoType),
    Optional(DynamoType),
}

pub fn possibly_optional_dynamo_type(ty: &syn::Type) -> PossiblyOptionalDynamoType {
    if matches_type(ty, "Option") {
        if let syn::Type::Path(ref p) = ty {
            if let AngleBracketed(AngleBracketedGenericArguments { args, .. }) = &p.path.segments[0].arguments {
                return match &args[0] {
                    syn::GenericArgument::Type(t) => PossiblyOptionalDynamoType::Optional(dynamo_type(t)),
                    _ => unreachable!("Option should have an inner type")
                }
            }
        }
        unreachable!("Option should have inner type");
    } else {
        PossiblyOptionalDynamoType::Normal(dynamo_type(ty))
    }
}

pub fn dynamo_type(ty: &syn::Type) -> DynamoType {
    let vec_nums: Vec<String> = ALL_NUMERIC_TYPES_AS_STRINGS.to_vec().iter().map(|num| format!("Vec{}", num)).collect();
    if matches_any_type(ty, vec_nums.iter().map(|s| &s as &str).collect()) {
        DynamoType::NumberList
    } else if matches_type(ty, "VecString") { // what about Vec&str?
        DynamoType::StringList
    } else if matches_type(ty, "HashMapStringString") {
        DynamoType::Map
    } else if matches_any_type(ty, ALL_NUMERIC_TYPES_AS_STRINGS.to_vec()) {
        DynamoType::Number
    } else if matches_type(ty, "bool") {
        DynamoType::Boolean
    } else {
        DynamoType::String
    }
}

pub fn scalar_dynamo_type(ty: &syn::Type) -> DynamoScalarType {
    if matches_any_type(ty, ALL_NUMERIC_TYPES_AS_STRINGS.to_vec()) {
        DynamoScalarType::Number
    } else if matches_type(ty, "bool") {
        DynamoScalarType::Boolean
    } else {
        DynamoScalarType::String
    }
}

pub fn get_ident_and_type_of_field_annotated_with<'a>(fields: &'a Punctuated<Field, Comma>, name: &'a str) -> Option<(&'a Ident, &'a syn::Type)> {
    fields.iter()
        .filter(|f| get_attribute(f, name).is_some())
        .map(|f| (f.ident.as_ref().unwrap(), &f.ty))
        .next()
}

fn get_attribute<'a>(f: &'a syn::Field, name: &'a str) -> Option<&'a syn::Attribute> {
    for attr in &f.attrs {
        if attr.path.segments.len() == 1 && attr.path.segments[0].ident == name.to_string() {
            return Some(attr);
        }
    }
    None
}

pub fn get_relevant_field_info<'a>(f: &'a Field) -> (&'a Ident, String, &syn::Type) {
    let name = &f.ident.as_ref().unwrap();
    let name_as_string = name.to_string();
    let field_type = &f.ty;
    (name, name_as_string, field_type)
}

pub fn matches_any_type<'a>(ty: &'a syn::Type, type_names: Vec<&str>) -> bool {
    type_names.iter().any(|v| matches_type(ty, v))
}

pub fn matches_type<'a>(ty: &'a syn::Type, type_name: &str) -> bool {
    // println!("IN HERE WITH {:?}", ty);

    if let syn::Type::Path(ref p) = ty {
        let mut first_match = p.path.segments[0].ident.to_string();

        if first_match == "Vec" {
            if let AngleBracketed(AngleBracketedGenericArguments { args, .. }) = &p.path.segments[0].arguments {
                let addition = args.iter().next().and_then(|rabbit_hole| {
                    match rabbit_hole {
                        syn::GenericArgument::Type(syn::Type::Path(ty)) => {
                            Some(ty.path.segments[0].ident.to_string())
                        }
                        _ => None,
                    }
                });
                first_match = format!("{}{}", first_match, addition.unwrap_or("".to_string()));
            }
        }

        if first_match == "HashMap" {
            if let AngleBracketed(AngleBracketedGenericArguments { args, .. }) = &p.path.segments[0].arguments {
                let map_args: Vec<String> = args.iter().filter_map(|rabbit_hole| {
                    match rabbit_hole {
                        syn::GenericArgument::Type(syn::Type::Path(ty)) => {
                            Some(ty.path.segments[0].ident.to_string())
                        }
                        _ => None,
                    }
                }).collect();
                first_match = format!("{}{}", first_match, map_args.join(""));
            }
        }

        return first_match == type_name.to_string();
    }
    false
}

pub fn get_macro_attribute(attrs: &Vec<Attribute>, attribute_name: &str) -> Vec<String> {
    attrs
        .into_iter()
        .filter(|attribute| attribute.path.is_ident(attribute_name))
        .flat_map(|attribute| {
            attribute.tokens.clone().into_iter().flat_map(|t| {
                match t {
                    Group(g) => {
                        g.stream().into_iter().filter_map(|s| {
                            match s {
                                Literal(l) => {
                                    Some(l.to_string())
                                }
                                _ => None
                            }
                        }).collect()
                    }
                    _ => vec![]
                }
            })
                .collect::<Vec<String>>()
        })
        .map(|att| att.replace("\"", "")) // caused by the to string, but perhaps a better way to get rid of it
        .collect()
}

pub fn tokenstream_or_empty_if_exclusion(stream: TokenStream2, method_name: &str, exclusions: &Vec<String>) -> TokenStream2 {
    if exclusions.contains(&method_name.to_string()) {
        quote! {}
    } else {
        stream
    }
}
