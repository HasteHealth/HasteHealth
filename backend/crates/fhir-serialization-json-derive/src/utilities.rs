use core::panic;
use quote::ToTokens;
use syn::{
    Attribute, Expr, Field, Ident, Lit, Meta, MetaList, Token, Type, punctuated::Punctuated,
};

/// Use rename_field attribute if present else use the struct name
pub fn get_field_name(field: &Field) -> String {
    get_attribute_value(&field.attrs, "rename_field")
        .unwrap_or_else(|| field.ident.as_ref().unwrap().to_string())
}

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

pub fn get_field_type(field: &Field) -> proc_macro2::Ident {
    match &field.ty {
        Type::Path(path) => path.path.segments.first().unwrap().ident.clone(),
        _ => panic!("Unsupported field type for serialization"),
    }
}

pub fn is_optional_field(field: &Field) -> bool {
    let field_type = get_field_type(field);
    if field_type == "Option" { true } else { false }
}

pub fn is_type_choice_field(field: &Field) -> bool {
    is_attribute_present(&field.attrs, "type_choice_variants")
}

pub fn is_attribute_present(attrs: &[Attribute], attribute: &str) -> bool {
    attrs.iter().any(|attr| attr.path().is_ident(attribute))
}

pub fn get_inner_type_if_optional(type_: &Type) -> Type {
    if let Type::Path(path) = type_ {
        if let Some(inner_type) = path.path.segments.first() {
            if inner_type.ident == "Option" {
                if let syn::PathArguments::AngleBracketed(args) = &inner_type.arguments {
                    if let Some(syn::GenericArgument::Type(ty)) = args.args.first() {
                        return ty.clone();
                    }
                }
            }
        }
    }
    type_.clone()
}

pub fn get_inner_type_if_vector_or_optional_or_box(type_: &Type) -> Type {
    if let Type::Path(path) = type_ {
        if let Some(inner_type) = path.path.segments.first() {
            if inner_type.ident == "Option"
                || inner_type.ident == "Vec"
                || inner_type.ident == "Box"
            {
                if let syn::PathArguments::AngleBracketed(args) = &inner_type.arguments {
                    if let Some(syn::GenericArgument::Type(ty)) = args.args.first() {
                        return get_inner_type_if_vector_or_optional_or_box(ty);
                    }
                }
            }
        }
    }
    type_.clone()
}

/// Renders a `Type` as a path expression usable before `::method(...)`, inserting the
/// turbofish (`::<...>`) on any segment carrying generic arguments. `syn`/`quote` render
/// generics the type-position way (`Foo<Bar>`) by default, which is invalid in expression
/// position (e.g. `Vec<T>::method()` doesn't parse; it must be `Vec::<T>::method()`).
pub fn type_as_turbofish_tokens(type_: &Type) -> proc_macro2::TokenStream {
    use quote::quote;

    if let Type::Path(type_path) = type_ {
        let mut tokens = proc_macro2::TokenStream::new();

        if let Some(qself) = &type_path.qself {
            let ty = &qself.ty;
            tokens.extend(quote! { <#ty> :: });
        }

        for (i, segment) in type_path.path.segments.iter().enumerate() {
            if i > 0 {
                tokens.extend(quote! { :: });
            }

            let ident = &segment.ident;
            match &segment.arguments {
                syn::PathArguments::AngleBracketed(args) => {
                    tokens.extend(quote! { #ident::#args });
                }
                syn::PathArguments::Parenthesized(args) => {
                    tokens.extend(quote! { #ident #args });
                }
                syn::PathArguments::None => {
                    tokens.extend(quote! { #ident });
                }
            }
        }

        tokens
    } else {
        quote::quote! { #type_ }
    }
}

pub fn is_type_string(type_: &Type) -> bool {
    let inner_type = get_inner_type_if_vector_or_optional_or_box(type_);

    match inner_type {
        Type::Path(path) => path
            .path
            .segments
            .last()
            .map(|segment| segment.ident == "String")
            .unwrap_or(false),
        _ => false,
    }
}

// Should return if it's a vector even if Option<Vec<T>>
pub fn is_vector(field: &Field) -> bool {
    let field_type = get_field_type(field);
    if field_type == "Vec" {
        true
    } else if field_type == "Option" {
        // Check if it's an Option<Vec<T>>
        let inner_type = get_inner_type_if_optional(&field.ty);

        if let Type::Path(path) = inner_type {
            if let Some(inner_type) = path.path.segments.first() {
                return inner_type.ident == "Vec";
            }
        }

        false
    } else {
        false
    }
}

#[derive(Debug)]
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

#[derive(Debug)]
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

#[allow(unused)]
pub fn get_reference_target_attribute(attrs: &[Attribute]) -> Vec<String> {
    if let Some(attribute_list) = get_attribute_list(attrs, "reference") {
        let parsed_arguments = attribute_list
            .parse_args_with(Punctuated::<Expr, Token![,]>::parse_terminated)
            .unwrap();

        let mut targets: Vec<String> = Vec::new();

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
                                "targets" => targets = variants,

                                _ => panic!(
                                    "reference must be in format like #[reference(target =[\"Resource\"])]"
                                ),
                            }
                        }
                        (k, v) => {
                            println!("{:?}", k);
                            panic!(
                                "reference must be in format like #[reference(target =[\"Resource\"])] but found {:?} = {:?}",
                                k, v
                            );
                        }
                    }
                }
                _ => {
                    panic!("reference must be in format like #[reference(target =[\"Resource\"])]")
                }
            }
        }

        targets
    } else {
        vec![]
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

#[derive(Debug)]
pub enum TypeInformation {
    Primitive,
    TypeChoice(TypeChoiceAttribute),
    Complex,
}

#[derive(Debug)]
pub struct FieldInformation {
    pub ident: Ident,
    pub ty: Type,
    pub field_type: Type,
    pub field_name: String,
    pub type_info: TypeInformation,
    pub is_vector: bool,
    pub is_optional: bool,
    pub cardinality: Option<CardinalityAttribute>,
}

// Get the various metadata extracted from the field.
pub fn process_field(field: &Field) -> FieldInformation {
    let is_primitive = is_attribute_present(&field.attrs, "primitive");
    let type_choice_attr = get_type_choice_attribute(&field.attrs);
    let is_type_choice = type_choice_attr.is_some();

    FieldInformation {
        ident: field.ident.clone().unwrap(),
        ty: field.ty.clone(),
        field_type: get_inner_type_if_vector_or_optional_or_box(&field.ty),
        field_name: get_field_name(field),
        is_vector: is_vector(field),
        is_optional: is_optional_field(field),
        cardinality: get_cardinality_attributes(&field.attrs),

        type_info: if is_primitive {
            TypeInformation::Primitive
        } else if is_type_choice {
            TypeInformation::TypeChoice(type_choice_attr.unwrap())
        } else {
            TypeInformation::Complex
        },
    }
}
