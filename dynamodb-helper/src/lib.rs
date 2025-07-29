#![doc = include_str!("../README.md")]

extern crate core;

mod r#impl; // TODO rename

use crate::r#impl::*;
use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use syn::Data::Enum;
use syn::Data::Struct;
use syn::Data::Union;
use syn::DataStruct;
use syn::Fields::Named;
use syn::FieldsNamed;
use syn::{parse_macro_input, DeriveInput, Error};

#[proc_macro_derive(DynamoDb, attributes(partition, range, exclusion))]
pub fn create_dynamodb_helper(item: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(item as DeriveInput);
    let name = ast.ident;
    let helper_name = format!("{name}Db");
    let helper_ident = Ident::new(&helper_name, name.span());

    let fields = match ast.data {
        Struct(DataStruct {
            fields: Named(FieldsNamed { ref named, .. }),
            ..
        }) => named,
        Enum(_) => {
            return Error::new(
                name.span(),
                "DynamoDB macro cannot be used with an enum - use a struct instead".to_string(),
            )
            .into_compile_error()
            .into()
        }
        Union(_) => {
            return Error::new(
                name.span(),
                "DynamoDB macro cannot be used with a union - use a struct instead".to_string(),
            )
            .into_compile_error()
            .into()
        }
        _ => {
            return Error::new(
                name.span(),
                "DynamoDB macro can only be used with a struct with named fields".to_string(),
            )
            .into_compile_error()
            .into()
        }
    };

    let exclusion_list = get_macro_attribute(&ast.attrs, EXCLUSION_ATTRIBUTE_NAME);
    let exclusion_list_refs: Vec<&str> = exclusion_list.iter().map(|x| &**x).collect();

    let (get_error, get_by_partition_error, batch_get_error, scan_error, parse_error) = generate_error_names(&helper_ident);
    let errors = generate_helper_error(&helper_ident, &exclusion_list_refs);

    let partition_key_ident_and_type = match get_ident_and_type_of_field_annotated_with(fields, PARTITION_KEY_ATTRIBUTE_NAME) {
        Some(res) => res,
        None => {
            return Error::new(
                name.span(),
                "You need to define a partition key for your DynamoDB struct! Place the `#[partition]` attribute above the field that serves as your key.".to_string()
            )
            .into_compile_error()
            .into();
        }
    };

    match dynamo_type(partition_key_ident_and_type.1) {
        None => {
            return Error::new(
                partition_key_ident_and_type.0.span(),
                "DynamoDB only supports strings, numbers and booleans as keys".to_string()
            ).into_compile_error().into();
        }
        _ => {}
    }

    let range_key_ident_and_type = get_ident_and_type_of_field_annotated_with(fields, RANGE_KEY_ATTRIBUTE_NAME);

    let from_struct_for_hashmap = tokenstream_or_empty_if_no_put_methods(from_struct_for_hashmap(&name, fields), &exclusion_list_refs);

    let try_from_hashmap_for_struct =
        tokenstream_or_empty_if_no_retrieval_methods(try_from_hashmap_to_struct(&name, &parse_error, fields), &exclusion_list_refs);

    let new = tokenstream_or_empty_if_exclusion(new_method(&helper_ident), NEW_METHOD_NAME, &exclusion_list_refs);

    let build = tokenstream_or_empty_if_exclusion(build_method(&helper_ident), BUILD_METHOD_NAME, &exclusion_list_refs);

    let gets = tokenstream_or_empty_if_exclusion(
        get_methods(
            &name,
            &get_error,
            &get_by_partition_error,
            partition_key_ident_and_type,
            range_key_ident_and_type,
        ),
        GET_METHOD_NAME,
        &exclusion_list_refs,
    );

    let batch_get = tokenstream_or_empty_if_exclusion(
        batch_get(&name, &batch_get_error, partition_key_ident_and_type, range_key_ident_and_type),
        BATCH_GET_METHOD_NAME,
        &exclusion_list_refs,
    );

    let create_table = tokenstream_or_empty_if_exclusion(
        create_table_method(partition_key_ident_and_type, range_key_ident_and_type),
        CREATE_TABLE_METHOD_NAME,
        &exclusion_list_refs,
    );
    let delete_table = tokenstream_or_empty_if_exclusion(delete_table_method(), DELETE_TABLE_METHOD_NAME, &exclusion_list_refs);
    let put = tokenstream_or_empty_if_exclusion(put_method(&name), PUT_METHOD_NAME, &exclusion_list_refs);
    let batch_put = tokenstream_or_empty_if_exclusion(batch_put_method(&name), BATCH_PUT_METHOD_NAME, &exclusion_list_refs);
    let delete = tokenstream_or_empty_if_exclusion(
        delete_method(&name, partition_key_ident_and_type, range_key_ident_and_type),
        DELETE_METHOD_NAME,
        &exclusion_list_refs,
    );
    let scan = tokenstream_or_empty_if_exclusion(scan_method(&name, &scan_error), SCAN_METHOD_NAME, &exclusion_list_refs);

    let public_version = quote! {
        #from_struct_for_hashmap
        #try_from_hashmap_for_struct

        pub struct #helper_ident {
            pub client: aws_sdk_dynamodb::Client,
            pub table: String,
        }

        impl #helper_ident {
            #new
            #build

            #create_table
            #delete_table

            #put
            #gets
            #batch_get
            #batch_put
            #delete
            #scan
        }

        #errors
    };

    public_version.into()
}
