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

    let public_version = quote! {
        impl Into<std::collections::HashMap<String, aws_sdk_dynamodb::model::AttributeValue>> for #name {
            fn into(self) -> std::collections::HashMap<String, aws_sdk_dynamodb::model::AttributeValue> {
                let mut map = std::collections::HashMap::new();
                #(#hashmap_inserts)*
                map
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
            pub async fn get(&self, key: #partition_key_type) -> Result<aws_sdk_dynamodb::output::GetItemOutput, aws_sdk_dynamodb::types::SdkError<aws_sdk_dynamodb::error::GetItemError>> {
                self.client.get_item()
                    .table_name(&self.table)
                    .key(#partition_key_name, AttributeValue::S(key))
                    .send()
                    .await
            }
        }
    };

    public_version.into()
}

