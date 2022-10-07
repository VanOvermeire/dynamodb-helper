use quote::__private::Ident;
use syn::{AngleBracketedGenericArguments, Field};
use syn::PathArguments::AngleBracketed;
use syn::punctuated::{Punctuated};
use syn::token::Comma;

pub const ALL_NUMERIC_TYPES_AS_STRINGS: &'static [&'static str] = &["u8", "u16", "u32", "u64", "u128", "i8", "i16", "i32", "i64", "i128", "f32", "f64"];

pub enum DynamoTypes {
    Number,
    Boolean,
    StringList,
    NumberList,
    // TODO boolean list
    List, // L [ {"S": "Cookies"} , {"S": "Coffee"}, {"N": "3.14159"}]
    Map, // M {"Name": {"S": "Joe"}, "Age": {"N": "35"}}, pass as Hashmap String AttributeValue
    String,
    // StringSet,
    // NumberSet,
    // Binary,
    // BinarySet,
    // Null, // = Null(bool)
}

pub enum DynamoScalarType {
    Number,
    String,
    Boolean,
}

pub fn dynamo_type(typez: &syn::Type) -> DynamoTypes {
    let vec_nums: Vec<String> = ALL_NUMERIC_TYPES_AS_STRINGS.to_vec().iter().map(|num| format!("Vec{}", num)).collect();
    if matches_any_type(typez, vec_nums.iter().map(|s| &s as &str).collect()) {
        DynamoTypes::NumberList
    } else if matches_type(typez, "VecString") { // what about Vec&str?
        DynamoTypes::StringList
    } else if matches_any_type(typez, ALL_NUMERIC_TYPES_AS_STRINGS.to_vec()) {
        DynamoTypes::Number
    } else if matches_type(typez, "bool") {
        DynamoTypes::Boolean
    } else {
        DynamoTypes::String
    }
}

pub fn scalar_dynamo_type(typez: &syn::Type) -> DynamoScalarType {
    if matches_any_type(typez, ALL_NUMERIC_TYPES_AS_STRINGS.to_vec()) {
        DynamoScalarType::Number
    } else if matches_type(typez, "bool") {
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
// hashmap string u32
// Path(TypePath { qself: None, path: Path { leading_colon: None, segments: [PathSegment { ident: Ident { ident: "HashMap", span: #0 bytes(1106..1113) }, arguments: AngleBracketed(AngleBracketedGenericArguments { colon2_token: None, lt_token: Lt, args: [Type(Path(TypePath { qself: None, path: Path { leading_colon: None, segments: [PathSegment { ident: Ident { ident: "String", span: #0 bytes(1114..1120) }, arguments: None }] } })), Comma, Type(Path(TypePath { qself: None, path: Path { leading_colon: None, segments: [PathSegment { ident: Ident { ident: "u32", span: #0 bytes(1122..1125) }, arguments: None }] } }))], gt_token: Gt }) }] } })
pub fn matches_type<'a>(ty: &'a syn::Type, type_name: &str) -> bool {
    if let syn::Type::Path(ref p) = ty {
        let mut first_match = p.path.segments[0].ident.to_string();

        if first_match == "Vec" || first_match == "HashMap" {
            if let AngleBracketed(AngleBracketedGenericArguments { args, .. }) = &p.path.segments[0].arguments {
                let addition = args.iter().next().and_then(|rabbit_hole| {
                        match rabbit_hole {
                            syn::GenericArgument::Type(syn::Type::Path(ty)) => Some(ty.path.segments[0].ident.to_string()),
                            _ => None,
                        }
                    });
                first_match = format!("{}{}", first_match, addition.unwrap_or("".to_string()));
            }
        }
        return first_match == type_name.to_string()
    }
    false
}
