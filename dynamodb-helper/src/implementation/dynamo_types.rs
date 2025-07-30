use crate::implementation::{matches_any_type, matches_type, ALL_NUMERIC_TYPES_AS_STRINGS};
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::spanned::Spanned;
use syn::PathArguments::AngleBracketed;
use syn::{AngleBracketedGenericArguments, Error, Type};

// in DynamoDB you wrap your values in the right 'attribute values', like N for numbers
// see for example https://docs.aws.amazon.com/amazondynamodb/latest/APIReference/API_AttributeValue.html
// so we need to find the right attribute value for the given input

#[derive(Debug)]
pub enum PossiblyOptionalDynamoType {
    Normal(IterableDynamoType),
    Optional(IterableDynamoType),
}

impl TryFrom<&Type> for PossiblyOptionalDynamoType {
    type Error = Error;

    fn try_from(value: &Type) -> Result<Self, Self::Error> {
        if matches_type(value, "Option") {
            if let Type::Path(ref p) = value {
                if let AngleBracketed(AngleBracketedGenericArguments { args, .. }) = &p.path.segments[0].arguments {
                    return match &args[0] {
                        syn::GenericArgument::Type(t) => Ok(PossiblyOptionalDynamoType::Optional(IterableDynamoType::try_from(t)?)),
                        _ => Err(Error::new(
                            value.span(),
                            "Expected this option type to have an inner type".to_string(),
                        )),
                    };
                }
            }
            Err(Error::new(
                value.span(),
                "Expected this option type to have an inner type".to_string(),
            ))
        } else {
            Ok(PossiblyOptionalDynamoType::Normal(IterableDynamoType::try_from(value)?))
        }
    }
}

#[derive(Debug)]
pub enum IterableDynamoType {
    Simple(DynamoType),
    List(DynamoType),
    Map(DynamoType, DynamoType),
}

impl TryFrom<&Type> for IterableDynamoType {
    type Error = Error;

    fn try_from(value: &Type) -> Result<Self, Self::Error> {
        if let Type::Path(ref p) = value {
            let first_match = p.path.segments[0].ident.to_string();

            if first_match == "Vec" {
                if let AngleBracketed(AngleBracketedGenericArguments { args, .. }) = &p.path.segments[0].arguments {
                    return match &args[0] {
                        syn::GenericArgument::Type(t) => Ok(IterableDynamoType::List(
                            DynamoType::from(t).ok_or(Error::new(value.span(), "Did not find a valid DynamoDB type for Vec's inner value".to_string()))?,
                        )),
                        _ => Err(Error::new(
                            value.span(),
                            "Vec should have an inner type - but we did not find one".to_string(),
                        )),
                    };
                }
            } else if first_match == "HashMap" {
                if let AngleBracketed(AngleBracketedGenericArguments { args, .. }) = &p.path.segments[0].arguments {
                    let map_args: Vec<Option<&Type>> = args
                        .iter()
                        .map(|rabbit_hole| match rabbit_hole {
                            syn::GenericArgument::Type(t) => Some(t),
                            _ => None,
                        })
                        .collect();

                    let map_key = map_args[0].ok_or(Error::new(value.span(), "Expected HashMap to have a key argument"))?;
                    let map_value = map_args[1].ok_or(Error::new(value.span(), "Expected HashMap to have a value argument"))?;

                    return Ok(IterableDynamoType::Map(
                        DynamoType::from(map_key)
                            .ok_or(Error::new(value.span(), "Did not find a valid DynamoDB type".to_string()))?,
                        DynamoType::from(map_value)
                            .ok_or(Error::new(value.span(), "Did not find a valid DynamoDB type".to_string()))?,
                    ));
                }
            }
        }
        Ok(IterableDynamoType::Simple(
            DynamoType::from(value).ok_or(Error::new(value.span(), "Did not find a valid DynamoDB type".to_string()))?,
        ))
    }
}

#[derive(Debug)]
pub enum DynamoType {
    Number,
    String,
    Boolean,
}

impl DynamoType {
    pub fn from(ty: &Type) -> Option<Self> {
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

    pub fn attribute_type_value(&self, name_of_attribute: Ident) -> TokenStream {
        match self {
            DynamoType::String => {
                quote! {
                    aws_sdk_dynamodb::types::AttributeValue::S(#name_of_attribute)
                }
            }
            DynamoType::Number => {
                quote! {
                    aws_sdk_dynamodb::types::AttributeValue::N(#name_of_attribute.to_string())
                }
            }
            DynamoType::Boolean => {
                quote! {
                    aws_sdk_dynamodb::types::AttributeValue::Bool(#name_of_attribute)
                }
            }
        }
    }

    pub fn scalar_attribute_type(&self) -> TokenStream {
        match self {
            DynamoType::String => {
                quote! {
                    aws_sdk_dynamodb::types::ScalarAttributeType::S
                }
            }
            DynamoType::Number => {
                quote! {
                    aws_sdk_dynamodb::types::ScalarAttributeType::N
                }
            }
            DynamoType::Boolean => {
                quote! {
                    aws_sdk_dynamodb::types::ScalarAttributeType::B
                }
            }
        }
    }
}
