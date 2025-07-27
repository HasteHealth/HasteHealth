#![allow(unused)]
use std::{cell::RefCell, rc::Rc};

use indexmap::IndexMap;
use proc_macro2::TokenStream;
use quote::quote;
use serde_json::Value;

use crate::{
    traversal,
    utilities::{extract, generate, load},
};

type NestedTypes = IndexMap<String, TokenStream>;

fn process_leaf(sd: &Value, element: &Value, types: Rc<RefCell<NestedTypes>>) {}

fn process_complex(
    sd: &Value,
    element: &Value,
    children: Vec<&Value>,
    serde_impls: Rc<RefCell<NestedTypes>>,
) {
    let struct_name = generate::struct_name(sd, element);
    for child_elem in children {}

    quote! { impl Deserialization for #struct_name {
        fn serialize(&self) -> String {
            serde_json::to_string(self).unwrap()
        }
    }};
}

fn create_visitor<'a>(
    sd: &'a Value,
    nested_serdes: Rc<RefCell<NestedTypes>>,
) -> impl FnMut(&'a Value, Vec<&'a Value>, usize) -> &'a Value {
    move |element: &'a Value, children: Vec<&'a Value>, _index: usize| -> &'a Value {
        if children.len() == 0 {
            element
        } else {
            process_complex(&sd, element, children, nested_serdes.clone());
            element
        }
    }
}

fn generate_deserialize_for_sd(sd: &Value) -> Result<TokenStream, String> {
    let nested_serdes = Rc::new(RefCell::new(IndexMap::<String, TokenStream>::new()));

    let mut visitor = create_visitor(sd, nested_serdes.clone());

    traversal::traversal(sd, &mut visitor)?;

    let binding = nested_serdes.borrow_mut();
    let types_generated = binding.values();

    let generated_code = quote! {
        #(#types_generated)*
    };

    Ok(generated_code)
}

pub fn generate_deserialize_for_sds(
    file_path: &str,
    level: Option<&'static str>,
) -> Result<Vec<TokenStream>, String> {
    let json_data = load::load_from_file(file_path)?;
    let structure_definitions = load::get_structure_definitions(&json_data, level)?;

    let mut generated_code = vec![];

    for sd in structure_definitions {
        generated_code.push(generate_deserialize_for_sd(sd)?);
    }

    Ok(generated_code)
}

pub fn generate_fhir_types_from_files(
    file_paths: &Vec<String>,
    level: Option<&'static str>,
) -> Result<String, String> {
    let mut serde_generated_code = quote! {
        #![allow(non_snake_case)]
        use serde::{Deserialize};
    };

    for file_path in file_paths {
        let serde_code_for_sds = generate_deserialize_for_sds(file_path, level)?;

        serde_generated_code = quote! {
            #serde_generated_code
            #(#serde_code_for_sds)*
        }
    }

    Ok(quote! {
        #serde_generated_code
    }
    .to_string())
}
