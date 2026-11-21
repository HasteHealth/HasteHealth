//! Functions FHIR adds to `FHIRPath`: <https://hl7.org/fhir/R4/fhirpath.html#functions>.

use crate::{
    Cardinality, Config, Context, FHIRPathError, ResolvedValue, boolean_result,
    evaluate_expression, parser::FunctionInvocation, singleton, validate_arguments,
};
use haste_fhir_model::r4::{
    conversion::{PRIMITIVE_TYPES, STRING_TYPES, downcast_string},
    generated::types::Extension,
};
use haste_reflect::MetaValue;
use std::{future::Future, pin::Pin, sync::Arc};

pub type ServiceFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// A system and code pair. A bare code has no system.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Code {
    pub system: Option<String>,
    pub code: String,
}

/// Server-side lookups behind `memberOf`, `subsumes`, `subsumedBy`,
/// `conformsTo` and `resolve`. A `None` answer means the lookup is unknown.
pub trait FHIRServices: Send + Sync {
    fn member_of<'a>(
        &'a self,
        value_set: &'a str,
        code: &'a Code,
    ) -> ServiceFuture<'a, Option<bool>>;

    /// Whether `left` subsumes `right`.
    fn subsumes<'a>(&'a self, left: &'a Code, right: &'a Code) -> ServiceFuture<'a, Option<bool>>;

    fn conforms_to<'a>(
        &'a self,
        resource: &'a dyn MetaValue,
        profile: &'a str,
    ) -> ServiceFuture<'a, Option<bool>>;

    fn resolve<'a>(&'a self, reference: &'a str) -> ServiceFuture<'a, Option<ResolvedValue>>;
}

fn services(
    name: &str,
    config: Option<&Arc<Config<'_>>>,
) -> Result<Arc<dyn FHIRServices>, FHIRPathError> {
    config
        .and_then(|c| c.services.clone())
        .ok_or_else(|| FHIRPathError::InternalError(format!("{name} requires services in config")))
}

/// Evaluates a single string argument, `None` when it is empty.
async fn string_argument<'a>(
    name: &str,
    function: &FunctionInvocation,
    context: &Context<'a>,
    config: Option<Arc<Config<'a>>>,
) -> Result<Option<String>, FHIRPathError> {
    validate_arguments(&function.arguments, &Cardinality::One)?;
    let argument = evaluate_expression(&function.arguments[0], context.clone(), config).await?;
    singleton(name, &argument)?
        .map(|v| downcast_string(v).map_err(Into::into))
        .transpose()
}

pub(crate) async fn evaluate_extension<'a>(
    function: &FunctionInvocation,
    context: Context<'a>,
    config: Option<Arc<Config<'a>>>,
) -> Result<Context<'a>, FHIRPathError> {
    let Some(url) = string_argument("extension", function, &context, config).await? else {
        return Ok(context.new_context_from(vec![]));
    };

    let matches = context
        .values
        .iter()
        .filter_map(|value| value.get_field("extension"))
        .flat_map(MetaValue::flatten)
        .filter(|ext| {
            ext.as_any()
                .downcast_ref::<Extension>()
                .is_some_and(|ext| ext.url == url)
        })
        .collect();

    Ok(context.new_context_from(matches))
}

/// The underlying system value of a FHIR primitive, `None` when it has none.
fn primitive_value(value: &dyn MetaValue) -> Option<&dyn MetaValue> {
    let fhir_type = value.fhir_type();
    if !PRIMITIVE_TYPES.contains(fhir_type) {
        return None;
    }
    if fhir_type.starts_with("http://hl7.org/fhirpath/") {
        return Some(value);
    }
    value.get_field("value")
}

pub(crate) fn evaluate_has_value<'a>(
    function: &FunctionInvocation,
    context: &Context<'a>,
) -> Result<Context<'a>, FHIRPathError> {
    validate_arguments(&function.arguments, &Cardinality::Zero)?;

    let has_value = match context.values.as_slice() {
        [value] => primitive_value(*value).is_some(),
        _ => false,
    };
    Ok(boolean_result(context, has_value))
}

pub(crate) fn evaluate_get_value<'a>(
    function: &FunctionInvocation,
    context: &Context<'a>,
) -> Result<Context<'a>, FHIRPathError> {
    validate_arguments(&function.arguments, &Cardinality::Zero)?;

    let value = singleton("getValue", context)?.and_then(primitive_value);
    Ok(context.new_context_from(value.into_iter().collect()))
}

/// Elements allowed in narrative xhtml: <https://hl7.org/fhir/R4/narrative.html#xhtml>.
const ALLOWED_ELEMENTS: &[&str] = &[
    "a",
    "abbr",
    "acronym",
    "b",
    "big",
    "blockquote",
    "br",
    "caption",
    "cite",
    "code",
    "col",
    "colgroup",
    "dd",
    "dfn",
    "div",
    "dl",
    "dt",
    "em",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "hr",
    "i",
    "img",
    "kbd",
    "li",
    "ol",
    "p",
    "pre",
    "q",
    "samp",
    "small",
    "span",
    "strong",
    "sub",
    "sup",
    "table",
    "tbody",
    "td",
    "tfoot",
    "th",
    "thead",
    "tr",
    "tt",
    "ul",
    "var",
];

/// Whether every tag is an allowed element and no attribute is an event
/// handler or a `javascript:` url.
fn html_is_safe(html: &str) -> bool {
    let mut rest = html;
    while let Some(start) = rest.find('<') {
        rest = &rest[start + 1..];
        let Some(end) = rest.find('>') else {
            return false;
        };
        let tag = &rest[..end];
        rest = &rest[end + 1..];

        // Comments and processing instructions carry no markup of their own.
        if tag.starts_with("!--") || tag.starts_with('?') {
            continue;
        }

        let tag = tag.trim_start_matches('/').trim_end_matches('/');
        let mut pieces = tag.split_ascii_whitespace();
        let name = pieces.next().unwrap_or_default();
        if !ALLOWED_ELEMENTS
            .iter()
            .any(|e| e.eq_ignore_ascii_case(name))
        {
            return false;
        }

        let attributes = tag[name.len()..].to_ascii_lowercase();
        if attributes.contains("javascript:")
            || attributes
                .split_ascii_whitespace()
                .any(|attribute| attribute.starts_with("on"))
        {
            return false;
        }
    }
    true
}

pub(crate) fn evaluate_html_checks<'a>(
    function: &FunctionInvocation,
    context: &Context<'a>,
) -> Result<Context<'a>, FHIRPathError> {
    validate_arguments(&function.arguments, &Cardinality::Zero)?;

    let Some(value) = singleton("htmlChecks", context)? else {
        return Ok(context.new_context_from(vec![]));
    };
    let safe = value.fhir_type() == "xhtml" && html_is_safe(&downcast_string(value)?);
    Ok(boolean_result(context, safe))
}

/// Every system and code held by a code, Coding or `CodeableConcept`.
fn codes_of(value: &dyn MetaValue) -> Vec<Code> {
    let text = |field: &str| value.get_field(field).and_then(|v| downcast_string(v).ok());

    if STRING_TYPES.contains(value.fhir_type()) {
        return downcast_string(value)
            .map(|code| vec![Code { system: None, code }])
            .unwrap_or_default();
    }
    if let Some(codings) = value.get_field("coding") {
        return codings.flatten().into_iter().flat_map(codes_of).collect();
    }
    text("code")
        .map(|code| Code {
            system: text("system"),
            code,
        })
        .into_iter()
        .collect()
}

pub(crate) async fn evaluate_member_of<'a>(
    function: &FunctionInvocation,
    context: Context<'a>,
    config: Option<Arc<Config<'a>>>,
) -> Result<Context<'a>, FHIRPathError> {
    let services = services("memberOf", config.as_ref())?;
    let Some(value_set) = string_argument("memberOf", function, &context, config).await? else {
        return Ok(context.new_context_from(vec![]));
    };
    let Some(value) = singleton("memberOf", &context)? else {
        return Ok(context.new_context_from(vec![]));
    };

    let mut known = false;
    for code in codes_of(value) {
        match services.member_of(&value_set, &code).await {
            Some(true) => return Ok(boolean_result(&context, true)),
            Some(false) => known = true,
            None => {}
        }
    }
    Ok(if known {
        boolean_result(&context, false)
    } else {
        context.new_context_from(vec![])
    })
}

pub(crate) async fn evaluate_subsumes<'a>(
    function: &FunctionInvocation,
    context: Context<'a>,
    config: Option<Arc<Config<'a>>>,
) -> Result<Context<'a>, FHIRPathError> {
    let name = function.name.0.as_str();
    let services = services(name, config.as_ref())?;
    validate_arguments(&function.arguments, &Cardinality::One)?;

    let argument = evaluate_expression(&function.arguments[0], context.clone(), config).await?;
    let (Some(input), Some(other)) = (singleton(name, &context)?, singleton(name, &argument)?)
    else {
        return Ok(context.new_context_from(vec![]));
    };
    let (Some(input), Some(other)) = (
        codes_of(input).into_iter().next(),
        codes_of(other).into_iter().next(),
    ) else {
        return Ok(context.new_context_from(vec![]));
    };

    let result = if name == "subsumes" {
        services.subsumes(&input, &other).await
    } else {
        services.subsumes(&other, &input).await
    };
    Ok(result.map_or_else(
        || context.new_context_from(vec![]),
        |r| boolean_result(&context, r),
    ))
}

pub(crate) async fn evaluate_conforms_to<'a>(
    function: &FunctionInvocation,
    context: Context<'a>,
    config: Option<Arc<Config<'a>>>,
) -> Result<Context<'a>, FHIRPathError> {
    let services = services("conformsTo", config.as_ref())?;
    let Some(profile) = string_argument("conformsTo", function, &context, config).await? else {
        return Ok(context.new_context_from(vec![]));
    };
    let Some(resource) = singleton("conformsTo", &context)? else {
        return Ok(context.new_context_from(vec![]));
    };

    let result = services.conforms_to(resource, &profile).await;
    Ok(result.map_or_else(
        || context.new_context_from(vec![]),
        |r| boolean_result(&context, r),
    ))
}

/// Resolves references through the services when configured, otherwise the
/// input passes through unchanged for search indexing.
pub(crate) async fn evaluate_resolve<'a>(
    context: Context<'a>,
    config: Option<Arc<Config<'a>>>,
) -> Result<Context<'a>, FHIRPathError> {
    let Some(services) = config.as_ref().and_then(|c| c.services.clone()) else {
        return Ok(context);
    };

    let mut resolved = Vec::with_capacity(context.values.len());
    for value in &context.values {
        let reference = if STRING_TYPES.contains(value.fhir_type()) {
            downcast_string(*value).ok()
        } else {
            value
                .get_field("reference")
                .and_then(|r| downcast_string(r).ok())
        };
        if let Some(reference) = reference
            && let Some(found) = services.resolve(&reference).await
        {
            resolved.push(context.allocate(found));
        }
    }
    Ok(context.new_context_from(resolved))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FPEngine;
    use haste_fhir_model::r4::conversion::downcast_bool;
    use haste_fhir_model::r4::generated::{
        resources::Patient,
        types::{Coding, FHIRBoolean, FHIRCode, FHIRString},
    };

    struct Mock;

    impl FHIRServices for Mock {
        fn member_of<'a>(&'a self, _: &'a str, code: &'a Code) -> ServiceFuture<'a, Option<bool>> {
            Box::pin(async move { Some(code.code == "in") })
        }

        fn subsumes<'a>(&'a self, l: &'a Code, r: &'a Code) -> ServiceFuture<'a, Option<bool>> {
            Box::pin(async move { Some(l.code == "parent" && r.code == "child") })
        }

        fn conforms_to<'a>(
            &'a self,
            _: &'a dyn MetaValue,
            profile: &'a str,
        ) -> ServiceFuture<'a, Option<bool>> {
            Box::pin(async move { Some(profile == "http://example.org/ok") })
        }

        fn resolve<'a>(&'a self, reference: &'a str) -> ServiceFuture<'a, Option<ResolvedValue>> {
            Box::pin(async move {
                (reference == "Patient/1").then(|| ResolvedValue::Box(Box::new(Patient::default())))
            })
        }
    }

    fn patient() -> Patient {
        Patient {
            active: Some(Box::new(FHIRBoolean {
                value: Some(true),
                extension: Some(vec![Extension {
                    url: "http://example.org/a".to_string(),
                    ..Default::default()
                }]),
                ..Default::default()
            })),
            ..Default::default()
        }
    }

    async fn run(expression: &str, services: bool) -> Vec<String> {
        let patient = patient();
        let engine = FPEngine::new();
        let result = if services {
            let config = Arc::new(Config::builder().with_services(Arc::new(Mock)));
            engine
                .evaluate_with_config(expression, vec![&patient], config)
                .await
        } else {
            engine.evaluate(expression, vec![&patient]).await
        }
        .unwrap_or_else(|e| panic!("{expression} failed: {e}"));
        result
            .iter()
            .map(|v| match downcast_bool(v) {
                Ok(b) => b.to_string(),
                Err(_) => v.fhir_type().to_string(),
            })
            .collect()
    }

    #[tokio::test]
    async fn extension_filters_by_url() {
        let found = run("Patient.active.extension('http://example.org/a')", false).await;
        assert_eq!(found, ["Extension"]);
        assert!(
            run("Patient.active.extension('http://example.org/b')", false)
                .await
                .is_empty()
        );
    }

    #[tokio::test]
    async fn has_value_and_get_value() {
        assert_eq!(run("Patient.active.hasValue()", false).await, ["true"]);
        assert_eq!(run("Patient.birthDate.hasValue()", false).await, ["false"]);
        assert_eq!(run("Patient.active.getValue()", false).await, ["true"]);
        assert!(run("Patient.birthDate.getValue()", false).await.is_empty());
    }

    #[test]
    fn html_checks() {
        assert!(html_is_safe(
            "<div xmlns=\"x\"><p class=\"a\">hi<br/></p></div>"
        ));
        assert!(!html_is_safe("<div><script>x</script></div>"));
        assert!(!html_is_safe("<div><p onclick=\"x()\">hi</p></div>"));
        assert!(!html_is_safe(
            "<div><a href=\"javascript:x()\">hi</a></div>"
        ));
    }

    #[test]
    fn codes() {
        let coding = Coding {
            system: Some(Box::new("http://s".to_string().into())),
            code: Some(Box::new(FHIRCode {
                value: Some("c".to_string()),
                ..Default::default()
            })),
            ..Default::default()
        };
        assert_eq!(
            codes_of(&coding),
            [Code {
                system: Some("http://s".to_string()),
                code: "c".to_string()
            }]
        );
        let bare = FHIRString {
            value: Some("x".to_string()),
            ..Default::default()
        };
        assert_eq!(codes_of(&bare)[0].system, None);
    }

    #[tokio::test]
    async fn services_backed_functions() {
        assert_eq!(run("'in'.memberOf('http://vs')", true).await, ["true"]);
        assert_eq!(run("'out'.memberOf('http://vs')", true).await, ["false"]);
        assert_eq!(run("'parent'.subsumes('child')", true).await, ["true"]);
        assert_eq!(run("'child'.subsumedBy('parent')", true).await, ["true"]);
        assert_eq!(
            run("Patient.conformsTo('http://example.org/ok')", true).await,
            ["true"]
        );
        assert_eq!(run("'Patient/1'.resolve()", true).await, ["Patient"]);
        assert!(run("'Patient/2'.resolve()", true).await.is_empty());
    }

    #[tokio::test]
    async fn services_required() {
        let patient = patient();
        let result = FPEngine::new()
            .evaluate("'a'.memberOf('http://vs')", vec![&patient])
            .await;
        assert!(result.is_err());
    }
}
