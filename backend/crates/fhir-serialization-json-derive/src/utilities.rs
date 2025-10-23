use core::panic;
use quote::ToTokens;
use syn::{Attribute, Expr, Lit, Meta, MetaList, Token, punctuated::Punctuated};

pub fn get_attribute_value(attrs: &[Attribute], attribute: &str) -> Option<String> {
    attrs.iter().find_map(|attr| match &attr.meta {
        Meta::NameValue(name_value) => {
            if name_value.path.is_ident(attribute) {
                match &name_value.value {
                    Expr::Lit(lit) => match &lit.lit {
                        Lit::Str(lit) => Some(lit.value()),
                        _ => panic!("Expected a string literal"),
                    },
                    _ => panic!("Expected a string literal"),
                }
            } else {
                None
            }
        }
        _ => None,
    })
}

pub fn is_attribute_present(attrs: &[Attribute], attribute: &str) -> bool {
    attrs.iter().any(|attr| attr.path().is_ident(attribute))
}

pub struct CardinalityAttribute {
    pub min: Option<usize>,
    pub max: Option<usize>,
}

pub fn get_cardinality_attributes(attrs: &[Attribute]) -> Option<CardinalityAttribute> {
    if let Some(attribute_list) = get_attribute_list(attrs, "cardinality") {
        let mut cardinality_attribute = CardinalityAttribute {
            min: None,
            max: None,
        };

        let parsed_arguments = attribute_list
            .parse_args_with(Punctuated::<Expr, Token![,]>::parse_terminated)
            .unwrap();

        for expression in parsed_arguments {
            match expression {
                Expr::Assign(expr_assign) => {
                    match (expr_assign.left.as_ref(), expr_assign.right.as_ref()) {
                        (Expr::Path(path), Expr::Lit(value)) => {
                            match path.path.get_ident().to_token_stream().to_string().as_str() {
                                "min" => match &value.lit {
                                    Lit::Int(v) => {
                                        cardinality_attribute.min =
                                            Some(v.base10_parse::<usize>().unwrap());
                                    }
                                    _ => panic!(
                                        "cardinality min must be an integer like #[cardinality(min = 1, max = 1)]"
                                    ),
                                },
                                "max" => match &value.lit {
                                    Lit::Int(v) => {
                                        cardinality_attribute.max =
                                            Some(v.base10_parse::<usize>().unwrap());
                                    }
                                    _ => panic!(
                                        "cardinality min must be an integer like #[cardinality(min = 1, max = 1)]"
                                    ),
                                },
                                _ => panic!(
                                    "cardinality must be in format like #[cardinality(min = 1, max = 1)]"
                                ),
                            }
                        }
                        _ => {
                            panic!(
                                "cardinality must be in format like #[cardinality(min = 1, max = 1)]"
                            )
                        }
                    }
                }
                _ => {
                    panic!("cardinality must be in format like #[cardinality(min = 1, max = 1)]");
                }
            }
        }
        Some(cardinality_attribute)
    } else {
        None
    }
}

pub struct TypeChoiceAttribute {
    pub complex_variants: Vec<String>,
    pub primitive_variants: Vec<String>,
}
impl TypeChoiceAttribute {
    pub fn all(&self) -> Vec<String> {
        let mut all_variants = self.complex_variants.clone();
        all_variants.extend(self.primitive_variants.clone());
        // Extension variant.
        all_variants.extend(self.primitive_variants.iter().map(|v| format!("_{}", v)));
        all_variants
    }
}

pub fn get_type_choice_attribute(attrs: &[Attribute]) -> Option<TypeChoiceAttribute> {
    if let Some(attribute_list) = get_attribute_list(attrs, "type_choice_variants") {
        let mut typechoice_attributes = TypeChoiceAttribute {
            complex_variants: Vec::new(),
            primitive_variants: Vec::new(),
        };

        let parsed_arguments = attribute_list
            .parse_args_with(Punctuated::<Expr, Token![,]>::parse_terminated)
            .unwrap();

        if parsed_arguments.len() > 2 {
            panic!("Expected exactly 2 type choice variants");
        }

        for expression in parsed_arguments {
            match expression {
                Expr::Assign(expr_assign) => {
                    match (expr_assign.left.as_ref(), expr_assign.right.as_ref()) {
                        (Expr::Path(path), Expr::Array(type_choices)) => {
                            let variants: Vec<String> = type_choices
                                .elems
                                .iter()
                                .map(|lit| match lit {
                                    Expr::Lit(lit) => match &lit.lit {
                                        Lit::Str(lit_str) => lit_str.value(),
                                        _ => panic!("Expected a string literal for typechoice"),
                                    },
                                    _ => panic!("Expected a string literal for typechoice"),
                                })
                                .collect();
                            match path.path.get_ident().to_token_stream().to_string().as_str() {
                                "primitive" => typechoice_attributes.primitive_variants = variants,
                                "complex" => typechoice_attributes.complex_variants = variants,
                                _ => panic!(
                                    "typechoice must be in format like #[type_choice_variants(primitive =[\"valueString\"], complex = [\"valueAddress\"])"
                                ),
                            }
                        }
                        (k, v) => {
                            println!("{:?}", k);
                            panic!(
                                "typechoice must be in format like #[type_choice_variants(primitive =[\"valueString\"], complex = [\"valueAddress\"]) but found {:?} = {:?}",
                                k, v
                            );
                        }
                    }
                }
                _ => panic!(
                    "typechoice must be in format like #[type_choice_variants(primitive =[\"valueString\"], complex = [\"valueAddress\"])"
                ),
            }
        }

        Some(typechoice_attributes)
    } else {
        None
    }
}

fn get_attribute_list(attrs: &[Attribute], attribute: &str) -> Option<MetaList> {
    attrs.iter().find_map(|attr| match &attr.meta {
        Meta::List(meta_list) if meta_list.path.is_ident(attribute) => {
            Some(meta_list.clone())
            // let k = meta_list
            //     .parse_args_with(Punctuated::<LitStr, Token![,]>::parse_terminated)
            //     .unwrap();
            // k.into_iter()
            //     .map(|lit| lit.value())
            //     .collect::<Vec<String>>()
            //     .into()
        }
        _ => None,
    })
}
