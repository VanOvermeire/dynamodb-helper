extern crate core;

use quote::quote;
use proc_macro::{TokenStream};
use std::iter::Map;
use syn::{parse_macro_input, DeriveInput, Field};
use syn::Data::Struct;
use syn::DataStruct;
use syn::Fields::Named;
use syn::FieldsNamed;
use syn::Meta::NameValue;
use syn::MetaNameValue;
use syn::Lit;
use syn::punctuated::{Iter, Punctuated};
use syn::token::Comma;

// let mut table = "".to_string();
//
// for option in ast.attrs.into_iter() {
// let ident = &option.path.segments.first().unwrap().ident;
//
// if ident == "table" {
// let option = option.parse_meta().unwrap();
// match option {
// NameValue(MetaNameValue{ref lit, ..}) => {
// if let Lit::Str(lit) = lit {
// table = lit.value();
// }
// },
// _ =>
// }
// }
// }


// eprintln!("Something {:?}", fields);

fn get_attribute<'a>(f: &'a syn::Field, name: &'a str) -> Option<&'a syn::Attribute> {
    for attr in &f.attrs {
        if attr.path.segments.len() == 1 && attr.path.segments[0].ident == name.to_string() {
            return Some(attr);
        }
    }
    None
}

#[proc_macro_derive(DynamoDb, attributes(partition))]
pub fn create_dynamodb_helper(item: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(item as DeriveInput);
    let name = ast.ident;
    let helper_name = format!("{}Db", name);
    let helper_ident = syn::Ident::new(&helper_name, name.span());

    let fields = match ast.data {
        Struct(DataStruct { fields: Named(FieldsNamed { ref named, .. }), .. }) => named,
        _ => unimplemented!("Only works for structs"),
    };

    let partition_key_ident_and_type = fields.iter()
        .filter(|f| get_attribute(f, "partition").is_some())
        .map(|f| (f.ident.as_ref().unwrap(), &f.ty))
        .next()
        .expect("Partition key should be defined (with attribute #[partition])");
    let partition_key_name = partition_key_ident_and_type.0.to_string();
    let partition_key_type = partition_key_ident_and_type.1;

    eprintln!("{:?}", partition_key_ident_and_type);

    // TODO moving this into separate function not working? (not the right signature I guess)
    let hashmap_inserts = fields.iter().map(|f| {
        let name = &f.ident.as_ref().unwrap();
        let name_as_string = name.to_string();
        // TODO only works for String...

        quote! {
            map.insert(#name_as_string.to_string(), aws_sdk_dynamodb::model::AttributeValue::S(self.#name));
        }
    });
    let hashmap_inserts_new = fields.iter().map(|f| {
        let name = &f.ident.as_ref().unwrap();
        let name_as_string = name.to_string();
        // TODO only works for String...

        quote! {
            map.insert(#name_as_string.to_string(), aws_sdk_dynamodb::model::AttributeValue::S(input.#name));
        }
    });

    let struct_inserts = fields.iter().map(|f| {
        let name = &f.ident.as_ref().unwrap();
        let name_as_string = name.to_string();
        // TODO get and use type for right conversion

        quote! {
            #name: map.get(#name_as_string).map(|v| v.as_s().expect("Attribute value conversion to work")).map(|v| v.to_string()).expect("Value for struct property to be present"),
        }
    });

    let public_version = quote! {
        // TODO try from is perhaps more appropriate for these (and no need to panic in that case)
        impl From<#name> for std::collections::HashMap<String, aws_sdk_dynamodb::model::AttributeValue> {
            fn from(input: #name) -> Self {
                let mut map = std::collections::HashMap::new();
                #(#hashmap_inserts_new)*
                map
            }
        }
        // TODO do we need both of these?
        impl From<std::collections::HashMap<String, aws_sdk_dynamodb::model::AttributeValue>> for #name {
            fn from(map: std::collections::HashMap<String, aws_sdk_dynamodb::model::AttributeValue>) -> Self {
                #name {
                    #(#struct_inserts)*
                }
            }
        }

        pub struct #helper_ident {
            client: aws_sdk_dynamodb::Client,
            table: String,
        }

        impl #helper_ident {
            fn new(client: aws_sdk_dynamodb::Client, table: &str) -> Self {
                #helper_ident {
                    client,
                    table: table.to_string()
                }
            }

            pub async fn build(region: aws_sdk_dynamodb::Region, table: &str) -> Self {
                let region_provider = aws_config::meta::region::RegionProviderChain::first_try(region).or_default_provider();
                let shared_config = aws_config::from_env().region(region_provider).load().await;
                #helper_ident {
                    client: aws_sdk_dynamodb::Client::new(&shared_config),
                    table: table.to_string(),
                }
            }

            pub async fn put(&self, input: #name) -> Result<aws_sdk_dynamodb::output::PutItemOutput, aws_sdk_dynamodb::types::SdkError<aws_sdk_dynamodb::error::PutItemError>> {
                // println!("Putting stuff");
                // let temp: std::collections::HashMap<String, aws_sdk_dynamodb::model::AttributeValue> = input.into();
                self.client.put_item()
                    .table_name(self.table.to_string())
                    .set_item(Some(input.into()))
                    .send()
                    .await
            }

            // TODO we could transform get item output into something useful
            pub async fn get(&self, key: #partition_key_type) -> Result<#name, aws_sdk_dynamodb::types::SdkError<aws_sdk_dynamodb::error::GetItemError>> {
                let result = self.client.get_item()
                    .table_name(&self.table)
                    .key(#partition_key_name, AttributeValue::S(key))
                    .send()
                    .await?;
                let mappie = result.item.expect("Just temp"); // TODO
                Ok(#name::from(mappie))
            }
        }
    };

    public_version.into()
}

