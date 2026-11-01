//! `#[derive(OperationOutcomeError)]` for error enums. Each variant lists the
//! issues it answers with; the enum then converts into an
//! `OperationOutcomeError`, and a `#[from]` field makes it convert from the
//! error it wraps.

use haste_fhir_model::r4::generated::terminology::{IssueType, ValueSetDef};
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{format_ident, quote};
use syn::{
    Attribute, Data, DeriveInput, Fields, Ident, LitStr, Type, Variant, parse_macro_input,
    spanned::Spanned,
};

const SEVERITIES: [&str; 4] = ["fatal", "error", "warning", "information"];

/// One issue attribute: `#[error(code = "not-found", diagnostic = "...")]`.
/// `severity` and `code` are the names of the model's constructors.
struct Issue {
    severity: Ident,
    code: Ident,
    diagnostic: Option<LitStr>,
}

/// Parse an attribute as an issue, or `None` when it is some other attribute.
fn parse_issue(attr: &Attribute) -> syn::Result<Option<Issue>> {
    let Some(severity) = attr
        .path()
        .get_ident()
        .filter(|ident| SEVERITIES.contains(&ident.to_string().as_str()))
    else {
        return Ok(None);
    };

    let mut code = None;
    let mut diagnostic = None;
    attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("code") {
            let literal: LitStr = meta.value()?.parse()?;
            let value = literal.value();
            if IssueType::CODES.binary_search(&value.as_str()).is_err() {
                return Err(syn::Error::new(
                    literal.span(),
                    format!(
                        "'{value}' is not a FHIR issue type; expected one of {:?}",
                        IssueType::CODES
                    ),
                ));
            }
            code = Some(Ident::new(&value.replace('-', "_"), literal.span()));
        } else if meta.path.is_ident("diagnostic") {
            diagnostic = Some(meta.value()?.parse()?);
        } else {
            return Err(meta.error("expected `code` or `diagnostic`"));
        }
        Ok(())
    })?;

    let Some(code) = code else {
        return Err(syn::Error::new(attr.span(), "an issue needs a `code`"));
    };
    Ok(Some(Issue {
        severity: severity.clone(),
        code,
        diagnostic,
    }))
}

/// The `OperationOutcomeIssue` expression for an issue. A diagnostic is a
/// format string over the variant's fields as `arg0`, `arg1`, ...
fn issue_expr(issue: &Issue) -> proc_macro2::TokenStream {
    let Issue {
        severity,
        code,
        diagnostic,
    } = issue;
    let diagnostics = diagnostic.as_ref().map_or_else(
        || quote! { None },
        |text| {
            quote! {
                Some(Box::new(haste_fhir_model::r4::generated::types::FHIRString {
                    value: Some(format!(#text)),
                    ..Default::default()
                }))
            }
        },
    );
    quote! {
        haste_fhir_model::r4::generated::resources::OperationOutcomeIssue {
            severity: haste_fhir_model::r4::generated::terminology::IssueSeverity::#severity(),
            code: haste_fhir_model::r4::generated::terminology::IssueType::#code(),
            diagnostics: #diagnostics,
            ..Default::default()
        }
    }
}

/// A variant's match arm, plus the type of its `#[from]` field if it has one.
fn variant_arm(
    name: &Ident,
    variant: &Variant,
) -> syn::Result<(proc_macro2::TokenStream, Option<Type>)> {
    let issues = variant
        .attrs
        .iter()
        .map(parse_issue)
        .filter_map(Result::transpose)
        .collect::<syn::Result<Vec<_>>>()?;
    if issues.is_empty() {
        return Err(syn::Error::new(
            variant.span(),
            "a variant needs a `fatal`, `error`, `warning` or `information` attribute",
        ));
    }

    let fields: Vec<_> = match &variant.fields {
        Fields::Unit => Vec::new(),
        Fields::Unnamed(unnamed) => unnamed.unnamed.iter().collect(),
        Fields::Named(named) => {
            return Err(syn::Error::new(
                named.span(),
                "named fields are not supported; use a tuple variant",
            ));
        }
    };
    let args: Vec<Ident> = (0..fields.len()).map(|i| format_ident!("arg{i}")).collect();

    let from_positions: Vec<usize> = fields
        .iter()
        .enumerate()
        .filter(|(_, field)| field.attrs.iter().any(|attr| attr.path().is_ident("from")))
        .map(|(i, _)| i)
        .collect();
    if from_positions.len() > 1 {
        return Err(syn::Error::new(
            variant.span(),
            "only one field can be `#[from]`",
        ));
    }
    if !from_positions.is_empty() && fields.len() != 1 {
        return Err(syn::Error::new(
            variant.span(),
            "a variant with a `#[from]` field has no other fields",
        ));
    }
    let from = from_positions.first().map(|&i| fields[i].ty.clone());
    let source = from_positions.first().map_or_else(
        || quote! { None },
        |&i| {
            let arg = &args[i];
            quote! { Some(#arg.into()) }
        },
    );

    let ident = &variant.ident;
    let pattern = if args.is_empty() {
        quote! { #name::#ident }
    } else {
        quote! { #name::#ident(#(#args),*) }
    };
    let issues = issues.iter().map(issue_expr);
    // The diagnostics format the fields before the `#[from]` field moves into the source.
    let arm = quote! {
        #pattern => {
            let outcome = haste_fhir_model::r4::generated::resources::OperationOutcome {
                issue: vec![#(#issues),*],
                ..Default::default()
            };
            haste_fhir_operation_error::OperationOutcomeError::new(#source, outcome)
        }
    };
    Ok((arm, from))
}

fn expand(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let Data::Enum(data) = &input.data else {
        return Err(syn::Error::new(
            Span::call_site(),
            "OperationOutcomeError can only be derived for an enum",
        ));
    };
    let name = &input.ident;

    let mut arms = Vec::with_capacity(data.variants.len());
    let mut from_impls = Vec::new();
    for variant in &data.variants {
        let (arm, from) = variant_arm(name, variant)?;
        arms.push(arm);
        if let Some(ty) = from {
            let ident = &variant.ident;
            from_impls.push(quote! {
                impl From<#ty> for #name {
                    fn from(error: #ty) -> Self {
                        #name::#ident(error)
                    }
                }
            });
        }
    }

    // `track_caller` makes the error's origin the `?` that converted it.
    Ok(quote! {
        impl From<#name> for haste_fhir_operation_error::OperationOutcomeError {
            #[track_caller]
            fn from(value: #name) -> Self {
                match value {
                    #(#arms),*
                }
            }
        }
        #(#from_impls)*
    })
}

/// Derives `From<YourEnum> for OperationOutcomeError`.
///
/// Each variant carries one or more of `#[fatal(...)]`, `#[error(...)]`,
/// `#[warning(...)]` and `#[information(...)]`, with a `code` from the FHIR
/// `IssueType` value set and an optional `diagnostic` format string over the
/// variant's fields as `{arg0}`, `{arg1}`, ... A `#[from]` field becomes the
/// error's source and also generates `From<ThatError> for YourEnum`.
#[proc_macro_derive(
    OperationOutcomeError,
    attributes(fatal, error, warning, information, from)
)]
pub fn operation_error(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand(&input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
