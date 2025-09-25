use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, punctuated::Punctuated, Attribute, Data, DeriveInput, Expr, Ident, Lit, Meta, MetaList, Token, Type, Variant
};

static FATAL: &str = "fatal";
static ERROR: &str = "error";
static WARNING: &str = "warning";
static INFORMATION: &str = "information";

fn get_issue_list(attrs: &[Attribute]) -> Option<Vec<MetaList>> {
    let issues: Vec<MetaList> = attrs
        .iter()
        .filter_map(|attr| match &attr.meta {
            Meta::List(meta_list)
                if meta_list.path.is_ident(FATAL)
                    || meta_list.path.is_ident(ERROR)
                    || meta_list.path.is_ident(WARNING)
                    || meta_list.path.is_ident(INFORMATION) =>
            {
                Some(meta_list.clone())
            }
            _ => None,
        })
        .collect();

    Some(issues)
}

const CODES_ALLOWED: &[&str] = &[
    "invalid",
    "structure",
    "required",
    "value",
    "invariant",
    "security",
    "login",
    "unknown",
    "expired",
    "forbidden",
    "suppressed",
    "processing",
    "not-supported",
    "duplicate",
    "multiple-matches",
    "not-found",
    "deleted",
    "too-long",
    "code-invalid",
    "extension",
    "too-costly",
    "business-rule",
    "conflict",
    "transient",
    "lock-error",
    "no-store",
    "exception",
    "timeout",
    "incomplete",
    "throttled",
    "informational",
];

fn get_expr_string(expr: &Expr) -> Option<String> {
    if let Expr::Lit(lit) = expr {
        if let Lit::Str(lit_str) = &lit.lit {
            return Some(lit_str.value());
        }
    }
    None
}

#[derive(Clone)]
enum Severity {
    Fatal,
    Error,
    Warning,
    Information,
}

impl Into<String> for Severity {
    fn into(self) -> String {
        match self {
            Severity::Fatal => "fatal".to_string(),
            Severity::Error => "error".to_string(),
            Severity::Warning => "warning".to_string(),
            Severity::Information => "information".to_string(),
        }
    }
}

#[derive(Clone)]
struct SimpleIssue {
    severity: Severity,
    code: String,
    diagnostic: Option<String>,
}

fn get_severity(meta_list: &MetaList) -> Severity {
    if meta_list.path.is_ident("fatal") {
        Severity::Fatal
    } else if meta_list.path.is_ident("error") {
        Severity::Error
    } else if meta_list.path.is_ident("warning") {
        Severity::Warning
    } else if meta_list.path.is_ident("information") {
        Severity::Information
    } else {
        panic!(
            "Unknown severity type: {}",
            meta_list.path.get_ident().unwrap()
        );
    }
}

fn get_issue_attributes(attrs: &[Attribute]) -> Option<Vec<SimpleIssue>> {
    let mut simple_issue = vec![];
    if let Some(issue_attributes) = get_issue_list(&attrs) {
        for issues in issue_attributes {
            let parsed_arguments = issues
                .parse_args_with(Punctuated::<Expr, Token![,]>::parse_terminated)
                .unwrap();

            if parsed_arguments.len() > 2 {
                panic!("Expected exactly 2 arguments for issue attributes");
            }

            let severity = get_severity(&issues);
            let mut code: Option<String> = None;
            let mut diagnostic: Option<String> = None;

            for expression in parsed_arguments {
                match expression {
                    Expr::Assign(expr_assign) => match expr_assign.left.as_ref() {
                        Expr::Path(path) => {
                            match path.path.get_ident().unwrap().to_string().as_str() {
                                "code" => {
                                    code = get_expr_string(expr_assign.right.as_ref());
                                    if let Some(code) = code.as_ref() {
                                        if !CODES_ALLOWED.contains(&code.as_str()) {
                                            panic!(
                                                "Invalid code: '{}' Must be one of '{:?}'",
                                                code, CODES_ALLOWED
                                            );
                                        }
                                    }
                                }
                                "diagnostic" => {
                                    diagnostic = get_expr_string(expr_assign.right.as_ref());
                                }
                                _ => panic!(
                                    "Unknown error attribute: {}",
                                    path.path.get_ident().unwrap()
                                ),
                            }
                        }
                        _ => panic!("Expected an assignment expression"),
                    },
                    _ => {
                        panic!("Expected an assignment expression");
                    }
                }
            }

            simple_issue.push(SimpleIssue {
                severity,
                code: code.unwrap_or_else(|| "error".to_string()),
                diagnostic: diagnostic,
            });
        }
    }

    Some(simple_issue)
}

/// Derive the operatiooutcome issues from the
/// attributes issue, fatal, error, warning, information
fn derive_operation_issues(v: &Variant) -> proc_macro2::TokenStream {
    let issues = get_issue_attributes(&v.attrs).unwrap_or(vec![]);
    let invariant_operation_outcome_issues = issues.iter().map(|simple_issue: &SimpleIssue| {
        let severity_string: String = simple_issue.severity.clone().into();
        let severity = quote! { Box::new(oxidized_fhir_model::r4::generated::types::FHIRCode{
                id: None,
                extension: None,
                value: Some(#severity_string.to_string()),
            })
        };



        let diagnostic = if let Some(diagnostic) = simple_issue.diagnostic.as_ref() {
            quote! {
                Some(Box::new(oxidized_fhir_model::r4::generated::types::FHIRString{
                    id: None,
                    extension: None,
                    value: Some(format!(#diagnostic)),
                }))
            }
        } else {
            quote! {
                None
            }
        };

        let code_string = &simple_issue.code;
        let code = quote! {
            Box::new(oxidized_fhir_model::r4::generated::types::FHIRCode{
                id: None,
                extension: None,
                value: Some(#code_string.to_string()),
            })
        };

        quote! {
            oxidized_fhir_model::r4::generated::resources::OperationOutcomeIssue {
                id: None,
                extension: None,
                modifierExtension: None,
                severity: #severity,
                code: #code,
                details: None,
                diagnostics: #diagnostic,
                location: None,
                expression: None,
            }
        }
    });

    quote! {
        vec![#(#invariant_operation_outcome_issues),*]
    }
}

fn get_arg_identifier(i: usize) -> Ident {
    format_ident!("arg{}", i)
}

#[derive(Debug, Clone)]
struct FromInformation {
    variant: Variant,
    from: usize,
    error_type: Type,
}

/// Returns the argument identifier for the from variant.
/// This should be an error.
fn get_from_error(v: &Variant) -> Option<FromInformation> {
    let from_fields: Vec<FromInformation> = v.fields.iter().enumerate().filter_map(|(i, field)| {
        let from_attr = field.attrs.iter().find(|attr|{
            let p = attr.path().is_ident("from");
            p
        });

        if from_attr.is_some() {
            if from_attr.is_some() {
                Some(FromInformation {
                    variant: v.clone(),
                    from: i,
                    error_type: field.ty.clone()
                })
            } else {
                panic!("Expected a named field with 'from' attribute");
            }
        } else {
            None
        }
    }).collect();

    if from_fields.len() > 1 {
        panic!("Expected only one field with 'from' attribute");
    }

    from_fields.get(0).cloned()
}



/// Instantiate the arguments for the variant
/// This is used in formatting the error message.
/// Format is arg0, arg1, arg2, ...
fn instantiate_args( v: &Variant) -> proc_macro2::TokenStream {
    let arg_identifiers = (0..v.fields.len()).map(|i| get_arg_identifier(i)).collect::<Vec<_>>();
    if arg_identifiers.is_empty() {
        quote!{}
    }
    else {
        quote! {
            (#(#arg_identifiers),*)
        }
    }
}


#[proc_macro_derive(OperationOutcomeError, attributes(fatal, error, warning, information, from))]
pub fn operation_error(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    match input.data {
        Data::Enum(data) => {
            let name = input.ident;

            // Errors to implement from trait for.
            let mut from_information: Vec<FromInformation> = vec![];

            let variants: Vec<proc_macro2::TokenStream> = data.variants.iter().map(|v| {
                let ident = &v.ident;
                let op_issues = derive_operation_issues(v);
                let arg_instantiation = instantiate_args( v);

                let from_error = if let Some(from_info) = get_from_error(v) {
                    let arg_identifier = get_arg_identifier(from_info.from);
                    from_information.push(from_info);
                    quote!{ Some(#arg_identifier.into()) }
                } else {
                    quote! { None }
                };


                quote! {
                    #ident #arg_instantiation => {
                        
                        let mut operation_outcome = oxidized_fhir_model::r4::generated::resources::OperationOutcome::default();
                        operation_outcome.issue = #op_issues;
                        
                        oxidized_fhir_operation_error::OperationOutcomeError::new(#from_error, operation_outcome)
                    }
                }
            }).collect();

            let from_impl = from_information.into_iter().map(|from_info| {
                let error_type = &from_info.error_type;
                let from_variant = &from_info.variant.ident;

                quote! {
                    impl From<#error_type> for #name {
                        fn from(error: #error_type) -> Self {
                            #name::#from_variant(error)
                        }
                    }
                }
            });


            let expanded = quote! {
                impl From<#name> for oxidized_fhir_operation_error::OperationOutcomeError {
                    fn from(value: #name) -> Self {
                        match value {
                            #(#name::#variants),*
                        }
                    }
                }
                #(#from_impl)*
            };

            // println!("{}", expanded.to_string());

            expanded.into()
        }
        _ => {
            panic!("Can only derive OperationOutcomeError from an enum.")
        }
    }
}
