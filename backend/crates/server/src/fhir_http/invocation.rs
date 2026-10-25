/*
(operation)	        /$[name]                            POST	R	Parameters	N/A	N/A
                    /[type]/$[name]                     GET	N/A	N/A	N/A	N/A
                    /[type]/[id]/$[name]

A POST carries the input parameters as its body. A GET carries them as its
query string, typed against the operation's definition; the spec allows it only
for an operation with affectsState = false and primitive inputs.
*/
use crate::fhir_http::{FHIRRequestParsingError, HTTPRequest, get_parameters};
use axum::http::Method;
use haste_fhir_client::request::{
    FHIRInvokeInstanceRequest, FHIRInvokeSystemRequest, FHIRInvokeTypeRequest, FHIRRequest,
    InvocationRequest, Operation,
};
use haste_fhir_model::r4::generated::{
    resources::{OperationDefinition, Parameters, ResourceType},
    terminology::{BoundCode, IssueType, OperationParameterUse},
};
use haste_fhir_operation_error::OperationOutcomeError;
use serde_json::{Map, Number, Value, json};
use std::borrow::Cow;

/// Finds the OperationDefinition a GET invocation's query string is typed
/// against.
pub trait OperationDefinitionLookup: Sync {
    /// The definition of `request`'s operation, or an error if none defines it.
    fn operation_definition(
        &self,
        request: &InvocationRequest,
    ) -> impl Future<Output = Result<Cow<'static, OperationDefinition>, OperationOutcomeError>> + Send;
}

/// The operation `url_chunks` invokes, without its parameters, or `None` if
/// they name no operation.
pub fn target(url_chunks: &[String]) -> Result<Option<InvocationRequest>, FHIRRequestParsingError> {
    let parameters = Parameters::default();
    Ok(Some(match url_chunks {
        [operation] if operation.starts_with('$') => {
            InvocationRequest::System(FHIRInvokeSystemRequest {
                operation: Operation::new(operation),
                parameters,
            })
        }
        [resource_type, operation] if operation.starts_with('$') => {
            InvocationRequest::Type(FHIRInvokeTypeRequest {
                resource_type: ResourceType::try_from(resource_type.as_str())?,
                operation: Operation::new(operation),
                parameters,
            })
        }
        [resource_type, id, operation] if operation.starts_with('$') => {
            InvocationRequest::Instance(FHIRInvokeInstanceRequest {
                resource_type: ResourceType::try_from(resource_type.as_str())?,
                id: id.clone(),
                operation: Operation::new(operation),
                parameters,
            })
        }
        _ => return Ok(None),
    }))
}

/// `target` with the input parameters `req` carries.
pub async fn invocation(
    mut target: InvocationRequest,
    req: HTTPRequest<'_>,
    operations: &impl OperationDefinitionLookup,
) -> Result<FHIRRequest, OperationOutcomeError> {
    let parameters = match req.method {
        Method::POST => get_parameters(req)?,
        Method::GET => {
            query_parameters(&*operations.operation_definition(&target).await?, req.query)?
        }
        _ => Err(FHIRRequestParsingError::Unsupported(
            "Invalid method for invocation".to_string(),
        ))?,
    };
    let slot = match &mut target {
        InvocationRequest::Instance(target) => &mut target.parameters,
        InvocationRequest::Type(target) => &mut target.parameters,
        InvocationRequest::System(target) => &mut target.parameters,
    };
    *slot = parameters;
    Ok(FHIRRequest::Invocation(target))
}

fn invalid(message: String) -> OperationOutcomeError {
    OperationOutcomeError::error(IssueType::invalid(), message)
}

/// The primitive `raw` as the JSON value of a `type` element, or `None` if
/// `type` is not primitive.
fn primitive(kind: &str, raw: &str) -> Option<Result<Value, String>> {
    Some(match kind {
        "boolean" => raw
            .parse()
            .map(Value::Bool)
            .map_err(|_| "a boolean".to_string()),
        "integer" | "positiveInt" | "unsignedInt" => raw
            .parse::<i64>()
            .map(Value::from)
            .map_err(|_| "an integer".to_string()),
        "decimal" => raw
            .parse::<Number>()
            .map(Value::Number)
            .map_err(|_| "a decimal".to_string()),
        "string" | "uri" | "url" | "canonical" | "base64Binary" | "instant" | "date"
        | "dateTime" | "time" | "code" | "oid" | "id" | "markdown" | "uuid" => {
            Ok(Value::String(raw.to_string()))
        }
        _ => return None,
    })
}

/// `query` as the input parameters of `definition`: each query parameter
/// becomes a parameter of that name, repeats included, typed as the
/// definition declares it.
fn query_parameters(
    definition: &OperationDefinition,
    query: &str,
) -> Result<Parameters, OperationOutcomeError> {
    if definition
        .affectsState
        .as_ref()
        .and_then(|affects| affects.value)
        .unwrap_or(false)
    {
        return Err(OperationOutcomeError::error(
            IssueType::business_rule(),
            format!(
                "Operation '{}' affects state and must be invoked with POST.",
                definition.code.value.as_deref().unwrap_or_default()
            ),
        ));
    }

    let inputs = definition.parameter.as_deref().unwrap_or_default();
    let mut typed = Vec::new();

    for (name, raw) in url::form_urlencoded::parse(query.as_bytes()) {
        let input = inputs
            .iter()
            .find(|input| {
                input.use_ == OperationParameterUse::in_()
                    && input.name.value.as_deref() == Some(name.as_ref())
            })
            .ok_or_else(|| invalid(format!("Unknown parameter '{name}'.")))?;
        let kind = input.type_.as_ref().and_then(BoundCode::as_str);
        let value = kind
            .and_then(|kind| primitive(kind, &raw))
            .ok_or_else(|| {
                invalid(format!(
                    "Parameter '{name}' is not a primitive and cannot be sent in a query string."
                ))
            })?
            .map_err(|expected| invalid(format!("Parameter '{name}' must be {expected}.")))?;

        let mut entry = Map::new();
        entry.insert("name".to_string(), Value::String(name.into_owned()));
        let kind = kind.unwrap_or_default();
        entry.insert(
            format!("value{}{}", kind[..1].to_uppercase(), &kind[1..]),
            value,
        );
        typed.push(Value::Object(entry));
    }

    serde_json::from_value(json!({ "resourceType": "Parameters", "parameter": typed }))
        .map_err(|error| invalid(format!("Invalid query parameters: {error}")))
}
