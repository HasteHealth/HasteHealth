use fhir_model::r4::types::OperationOutcomeIssue;
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Attribute, Data, DeriveInput, Expr, Lit, Meta, MetaList, Token, parse_macro_input,
    punctuated::Punctuated,
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

// invalid
//     structure
//     required
//     value
//     invariant
// security
//     login
//     unknown
//     expired
//     forbidden
//     suppressed
// processing
//     not-supported
//     duplicate
//     multiple-matches
//     not-found
//         deleted
//     too-long
//     code-invalid
//     extension
//     too-costly
//     business-rule
//     conflict
// transient
//     lock-error
//     no-store
//     exception
//     timeout
//     incomplete
//     throttled
// informational

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

#[proc_macro_derive(Reflect, attributes(fatal, error, warning, information))]
pub fn operation_error(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    match input.data {
        Data::Enum(data) => {
            let name = input.ident;

            let variants = data.variants.iter().map(|v| {
                let ident = &v.ident;
                let issues = get_issue_attributes(&v.attrs);
                let issue_instantiation = if let Some(issues) = issues {
                    issues.iter().map(|simple_issue| {
                        let severity_string: String = simple_issue.severity.clone().into();
                        let severity = quote! { Box::new(fhir_model::r4::types::FHIRCode{
                                id: None,
                                Extension: None,
                                value: #severity_string.to_string(),
                            })
                        };

                        let diagnostic = if let Some(diagnostic) = simple_issue.diagnostic.as_ref()
                        {
                            quote! {
                                Some(Box::new(fhir_model::r4::types::FHIRCode{
                                    id: None,
                                    Extension: None,
                                    value: Some(#diagnostic.to_string()),
                                }))
                            }
                        } else {
                            quote! {
                                None
                            }
                        };

                        let code_string = &simple_issue.code;
                        let code = quote! {
                            Box::new(fhir_model::r4::types::FHIRCode{
                                id: None,
                                Extension: None,
                                value: #code_string.to_string(),
                            })
                        };

                        quote! {
                            OperationOutcomeIssue {
                                severity: #severity,
                                code: #code,
                                diagnostics: #diagnostic,
                            }
                        }
                    });
                    quote! {}
                } else {
                    quote! {}
                };

                quote! {
                    #ident => write!(f, stringify!(#ident)),
                }
            });

            let expanded = quote! {
                use fhir_operation_error::OperationError;
                impl From<#name> for OperationError {
                    fn from(value: #name) -> Self {
                        match value {
                            #(#name::#variants),*
                        }
                    }
                }
            };

            expanded.into()
        }
        _ => {
            panic!("Can only derive operationerror from an enum.")
        }
    }
}
